# Cross-Cycle Insights

## Recurring wins

- Scan-resistant insertion made the large early hit-rate gains: new one-touch items are evicted first (LIP quarter-depth, CLOCK cleared bit). Gains are biggest on wide buckets. Fixed quarter-depth LIP with BIP 1/32 MRU admission is still the best policy after five cycles.
- Reducing full-key compares is the one lever that moved get latency. A dense 1-byte fingerprint pre-filter, kept in its own array and scanned in vectorisable chunks, cut get and put latency at both p50 and p99. Misses dominate, so the miss scan is where the savings show up.
- Shrinking `Item` is the only thing that has moved metadata-size. The 24-byte layout kept hit rate exactly the same.
- Rewrites that keep the layout (fused rotation, narrow fields, per-bucket clock) reliably keep every hit-rate cell exactly the same. They are safe bases to stack speed work on.
- Removing shared or atomic state from the hot path pays off. The per-bucket non-atomic clock recovered the regression caused by the global atomic tick.
- Bounding worst-case work helps the put tail. Examples are the 16-slot sweep cap and a dense absence scan.

## Recurring failures

- Reuse and adaptation signals keep failing the hm_0 vs web_0 tradeoff:
  - BIP's blind period gives back web_0 scan resistance.
  - A fixed 16-key ghost ring misses hm_0's slow reuse.
  - A hashed ghost bit set fills up and resets all at once.
  - Adaptive depth based on hit position misreads aged-down items as pressure.
- Changing scan order or hit-time reordering does not move get p50. Self-organizing swaps make it worse. Hit-path writes must stay a single cheap store.
- A sparse side array of wide keys (a `u64` mirror) hurts locality. A second stream only pays off when it is much denser than the items.
- "Cheap" additions keep bringing hidden put-path costs: per-op string compares, dynamic policy calls, SipHash on misses, global atomics, unbounded sweeps. Each one cancels out the gain it was paired with.
- Extra work on the miss/eviction path stacks on top of the quarter-depth `Vec::insert` memmove instead of replacing it. Put p50 rises with almost every policy tweak.
- Flat metadata-size is not evidence that a change is free. The metric cannot see per-bucket state such as mirrors, ghosts, tags or bucket fields.

## Open questions

- Do the per-bucket clock and the fingerprint filter add up when combined? Can timestamp stores be skipped entirely when no active policy reads them?
- Does the fused evict-plus-insert rotation (with the victim index fixed at construction, using `ptr::copy`) cut put latency? It has never been measured cleanly, so it needs a same-session baseline.
- Can tags extend beyond the LRU/LIP path, for example to CLOCK? Would wider tags or SIMD on the mask cut the remaining false-positive compares?
- Is there any reuse signal worth another try? The only remaining candidate is a per-item probationary bit with symmetric grow and decay.
- How reliable is per-op-time between sessions? One cycle showed an apparent environment shift that was later traced to a real cause, so judge speed against a same-session parent.

## Per-policy notes

- **ClassicLRU / LIP / BIP:**
  - The quarter-depth insert is the current base.
  - Its memmove is the main put cost, and finding a cheaper insert-at-depth is still open.
  - BIP's 1/32 helps hm_0 on small buckets but costs a little on web_0.
  - Scaling the insertion depth by bucket width or by hit position has not helped.
  - Fingerprint tags currently apply only to this path.
- **CLOCK / GCLOCK:**
  - CLOCK's single-byte hit store is its latency advantage, and reordering on hits destroys it.
  - 2-bit GCLOCK with a 16-slot sweep cap and clear-on-insert gave a small hit-rate gain at every point and a better put tail, but a slower typical put.
  - Tags have not been tried on CLOCK yet.
- **Frequency-aware victim window:** v003's result was identical to the baseline, possibly because the harness never ran the overlaid eviction path. That has not been verified, so the idea has never been fairly judged.
- **Ghost / reuse-filter admission:** Both the fixed-size and the hashed variants failed. Deprioritize unless the design changes fundamentally.
