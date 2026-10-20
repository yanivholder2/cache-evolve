## Recurring wins

- Zero-metadata designs: bit-packing the second-hit admission flag into spare bits kept the hit-rate benefit, restored baseline item size, and improved latency. It is the only clean, repeatable win so far.
- Scan resistance (keeping one-hit-wonders out of the hot region) is the lever that pays off, whether through insert position or admission. Admission gating beats position tweaks.
- Latency gains appear only where work is removed (e.g. skipping remove-and-promote on a first hit), not where work is added.

## Recurring failures

- Insert-position tweaks (uniform MRU, cold/hot split, cold first admission, cold demotions): small gains, overlap with admission, and trade hit-rate against latency. This lever looks exhausted.
- Stacking overlapping levers (position plus admission): they target the same items, so gains do not add.
- Richer signals on top of the filter (2-bit counter, ref bit, skip-if-hot shortcut): negligible hit-rate gain and a latency loss, because each adds hot-path work. A branch that only sometimes succeeds is not free.
- "Winner" labels on tiny hit-rate gains are often within trace noise and should not be trusted.

## Open questions

- Are the small latency changes real or run-to-run noise? Re-measure before trusting them.
- Where does get/put time actually go (lookup, locking, bucket scan)? Latency work on those paths is untried.
- Do the levers matter on other traces (scan-heavy, frequency-skewed), or is saturation specific to the recency-skewed LIRS trace?
- Is the admission-plus-eviction space truly saturated, or only on this trace?

## Per-policy notes

- DASH (segment promotion/admission): the base to build on is the bit-packed second-hit filter at baseline item size. Spare bits can hold a counter or ref bit, so footprint is not a constraint, but hit-rate is saturated on LIRS. Next effort should go to latency on lookup, locking and bucket-scan paths, or to a different trace.
