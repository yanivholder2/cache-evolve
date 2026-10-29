# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-29T10:29:44+00:00

## timeline

### 10:30:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Scan-resistant insertion made the large early hit-rate gains: new one-touch items are evicted first (LIP quarter-depth, CLOCK cleared bit). Gains are biggest on wide buckets. Fixed quarter-depth LIP with BIP 1/32 MRU admission is still the best policy after five cycles.\n- Reducing full-key compares is the one lever that moved get latency. A dense 1-byte fingerprint pre-filter, kept in its own array and scanned in vectorisable chunks, cut get and put latency at both p50 and p99. Misses dominate, so the miss scan is where the savings show up.\n- Shrinking `Item` is the only thing that has moved metadata-size. The 24-byte layout kept hit rate exactly the same.\n- Rewrites that keep the layout (fused rotation, narrow fields, per-bucket clock) reliably keep every hit-rate cell exactly the same. They are safe bases to stack speed work on.\n- Removing shared or atomic state from the hot path pays off. The per-bucket non-atomic clock recovered the regression caused by the global atomic tick.\n- Bounding worst-case work helps the put tail. Examples are the 16-slot sweep cap and a dense absence scan.\n\n## Recurring failures\n\n- Reuse and adaptation signals keep failing the hm_0 vs web_0 tradeoff:\n  - BIP's blind period gives back web_0 scan resistance.\n  - A fixed 16-key ghost ring misses hm_0's slow reuse.\n  - A hashed ghost bit set fills up and resets all at once.\n  - Adaptive depth based on hit position misreads aged-down items as pressure.\n- Changing scan order or hit-time reordering does not move get p50. Self-organizing swaps make it worse. Hit-path writes must stay a single cheap store.\n- A sparse side array of wide keys (a `u64` mirror) hurts locality. A second stream only pays off when it is much denser than the items.\n- \"Cheap\" additions keep bringing hidden put-path costs: per-op string compares, dynamic policy calls, SipHash on misses, global atomics, unbounded sweeps. Each one cancels out the gain it was paired with.\n- Extra work on the miss/eviction path stacks on top of the quarter-depth `Vec::insert` memmove instead of replacing it. Put p50 rises with almost every policy tweak.\n- Flat metadata-size is not evidence that a change is free. The metric cannot see per-bucket state such as mirrors, ghosts, tags or bucket fields.\n\n## Open questions\n\n- Do the per-bucket clock and the fingerprint filter add up when combined? Can timestamp stores be skipped entirely when no active policy reads them?\n- Does the fused evict-plus-insert rotation (with the victim index fixed at construction, using `ptr::copy`) cut put latency? It has never been measured cleanly, so it needs a same-session baseline.\n- Can tags extend beyond the LRU/LIP path, for example to CLOCK? Would wider tags or SIMD on the mask cut the remaining false-positive compares?\n- Is there any reuse signal worth another try? The only remaining candidate is a per-item probationary bit with symmetric grow and decay.\n- How reliable is per-op-time between sessions? One cycle showed an apparent environment shift that was later traced to a real cause, so judge speed against a same-session parent.\n\n## Per-policy notes\n\n- **ClassicLRU / LIP / BIP:**\n  - The quarter-depth insert is the current base.\n  - Its memmove is the main put cost, and finding a cheaper insert-at-depth is still open.\n  - BIP's 1/32 helps hm_0 on small buckets but costs a little on web_0.\n  - Scaling the insertion depth by bucket width or by hit position has not helped.\n  - Fingerprint tags currently apply only to this path.\n- **CLOCK / GCLOCK:**\n  - CLOCK's single-byte hit store is its latency advantage, and reordering on hits destroys it.\n  - 2-bit GCLOCK with a 16-slot sweep cap and clear-on-insert gave a small hit-rate gain at every point and a better put tail, but a slower typical put.\n  - Tags have not been tried on CLOCK yet.\n- **Frequency-aware victim window:** v003's result was identical to the baseline, possibly because the harness never ran the overlaid eviction path. That has not been verified, so the idea has never been fairly judged.\n- **Ghost / reuse-filter admission:** Both the fixed-size and the hashed variants failed. Deprioritize unless the design changes fundamentally.\n"
}
```

### 10:30:13 chat

I wrote `cross-cycle-insights.md` in the cycle 0005 output dir, built fresh from the five-cycle ledger with the four required sections in order.

The main patterns across cycles:

- **Hit rate:** the big gains came early, from inserting new items where they are evicted first (LIP-style insertion, CLOCK's cleared bit). Fixed quarter-depth LIP with BIP 1/32 is still the best policy. Every attempt to make insertion react to observed reuse has failed to help both test workloads, hm_0 and web_0, at once. That covers BIP's fixed period, two ghost designs and adaptive depth.
- **Speed:** only two things have reliably helped. One is cutting the number of full-key compares with the dense 1-byte fingerprint filter. The other is removing shared or atomic state from the put path. Changing scan order and reordering on hits did nothing for get latency. Sparse wide-key mirrors made it worse.
- **Metadata-size:** shrinking `Item` is the only change that has ever moved it. The metric can't see per-bucket state, so a flat reading doesn't mean a change was free.
- **Open questions:**
  - Do the per-bucket clock and the fingerprint filter add up when combined?
  - Does the fused rotation cut put latency? It has never been measured cleanly.
  - Can tags extend to CLOCK?
  - The frequency-aware victim window was never fairly tested.


---

- finished: 2026-10-29T10:30:13+00:00
- duration: 28.73s
- status: ok
- input_tokens: 4
- output_tokens: 2195
- cache_read_tokens: 39423
- cache_write_tokens: 11208
- cost_usd: 0.141465
