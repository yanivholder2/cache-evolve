# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **Scan-resistant insertion (LIP quarter-depth + BIP).** This came out of cycle 1 and is still the best policy after many attempts to beat it. Most of its gain is on wide buckets.
- **Removing whole units of work beats re-implementing them.** Wins of this kind: the byte-tag pre-filter cut full-key compares (c5), the fused evict+insert rotation removed two memmoves (c6), lazy promotion skipped near-MRU hit moves (c9), and a single multiplicative hash replaced redundant SipHash (c9). Each one improved speed on more than one op or percentile.
- **Dense, same-stream side data.** Byte tags work where a `u64` key mirror failed. Density and locality decide whether a side array helps or hurts.
- **Look above the bucket.** Cycles 5–8 tuned in-bucket loops. The cost that turned out to matter in c9 was in the hashing and indexing path, which every op shares.
- **Mechanical rewrites are semantically safe.** Every layout or speed rewrite that was meant to be hit-rate-neutral matched all cells exactly. The risk is entirely in per-op time.
- **Shrinking `Item` is the only lever that moves metadata-size** (c4, 24-byte item).
- **Width-keyed static policy split.** It costs no metadata, and narrow cells stay bit-identical while wide buckets get their own setting.

## Recurring failures

- **Reuse/adaptation signals, five tries so far:** BIP blind period, a 16-key ghost ring, a hashed ghost bit set, adaptive depth from hit position, and a 1-bit second chance. None solved the hm_0-vs-web_0 tradeoff. Small windows miss slow reuse, dense filters saturate, and bits without their own decay go stale.
- **"Clever" replacements for simple inline loops:** `rotate_left(1)`, a variable-length `ptr::copy` on the hit path, and a SWAR tag scan. All were slower at p50 than the loops the compiler already handles well.
- **Reordering on hits or changing scan order:** reverse scan, swap-halfway, and self-organization did nothing for get p50 or made it worse.
- **Adding state or writes to skip cheap work:** a miss memo and writes on the get path. Both dirtied cache lines and grew the stride.
- **Extra work on the put-miss path:** mid-bucket insert, unbounded sweeps, per-miss hashing, a dynamic victim check, and a global atomic. Each one raised put latency.
- **Expecting a speed win from removing the atomic clock.** It is neutral on the tag/fused base. Keep the removal for correctness only.
- **BIP period tuning on wide buckets.** Going 1/32 → 1/64 → 1/128 trades web_0 against hm_0 roughly one-for-one, so there is no sweet spot. This probe is closed.
- **Trusting single-run per-op numbers.** Drift showed up in c4, c6, c7 and c8, including slowdowns in variants that only removed work.

## Open questions

- Do c9's single multiplicative hash and lazy promotion stack? Check with repeated same-session runs against both parents.
- Can the lazy-promotion window widen to `n/4` before it reaches the insertion region and starts changing eviction order?
- Where does the remaining cost sit: the full-key compare, the fused rotation's quarter-shift plus tag mirror, or bucket indexing (`%` vs mask)? Profile this before forming new hypotheses.
- Does stride cost anything on its own? This needs a pad-only control (80 → 88 bytes with no behaviour change).
- Can tags be extended to the CLOCK path?
- Is any reuse signal still worth a try? Only one with a per-item probationary bit, symmetric grow/decay, and placement outside the admission region.

## Per-policy notes

- **ClassicLRU / LIP / BIP (main line):** The current base is quarter-depth LIP with width-keyed BIP (1/32 narrow, 1/64 wide), byte tags, fused rotation, a clock-free 24-byte `Item`, an 80-byte stride, and now one multiplicative hash plus top-eighth lazy promotion. Keep the policy frozen apart from the promotion window.
- **CLOCK / GCLOCK:** Clearing the reference bit on insert gives scan resistance comparable to LIP. A 2-bit GCLOCK adds a little hit rate. Bound the sweep (16-slot cap) to keep the put tail down. The hit-path write must stay a single byte store, and any reordering on hits loses CLOCK's advantage. This policy has not been revisited since c3 and has no tag support.
- **Frequency-aware victim window (c1 v003):** Its result was bit-identical to the baseline, and it was never confirmed that the harness ran its path. Unresolved and deprioritized.
- **Ghost / second-chance variants:** Each improves one workload and costs the other, wide-bucket web_0 or small hm_0. Don't build another without independent decay.
- **Metric caveat (all policies):** metadata-size measures only the `Item` layout. Flat readings do not mean per-bucket state (tags, ghosts, counters) is free.
