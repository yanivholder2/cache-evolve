## Recurring wins

- The second-hit admission filter is the only real policy win. It was found in cycle 0001, made metadata-free by bit-packing in cycle 0002, and re-derived independently in cycle 0004. It keeps hit-rate gains, baseline item size and a latency benefit.
- Subtractive changes helped once. Removing bookkeeping the policy never reads (cycle 0004 v003) improved latency and hit-rate. The dead stores left after that were too few to measure (cycle 0005).
- Bit-packing works. Spare bits hold flags, counters and ref bits, so metadata footprint is not a constraint.

## Recurring failures

- Insert-position variants (uniform MRU, cold first admission, cold demotion) overlap with admission rather than stacking. They trade hit-rate against latency and look exhausted.
- Richer admission and recency signals (2-bit counter, ref bit, skip-if-hot guard) added almost no hit-rate, because the second-hit filter already removes most one-touch items. Each also cost latency.
- Blind hot-path edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skip, hoisting hash work out of the lock) have failed in four cycles in a row. They added branches or work and slightly worsened latency.
- "Winner" status is weak evidence. Hit-rate gains are within plausible trace noise, and latency regressions often appear on paths the change did not touch. That points to layout effects or measurement noise, and the noise floor has never been measured.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? Repeat baseline runs to find out.
- Where do get and put time actually go (lock acquisition, bucket scan, memory access)? Nothing has profiled this yet.
- Does a different trace (scans, frequency skew) show differences the LIRS trace hides? All hit-rate conclusions come from one recency-skewed trace.
- How does the cycle 0004 v003 minimal filter compare with the cycle 0002 v001 incumbent? How much of its gain is dead-bookkeeping removal and how much is the filter?
- Do structural changes to probing, bucket layout or locking help? Try them only if profiling points there.

## Per-policy notes

- DASH segment promotion and admission policy (the only policy explored so far):
  - Base: the minimal bit-packed second-hit filter with dead bookkeeping removed, at baseline item size.
  - Position levers, admission signals and eviction-order levers all look saturated on the LIRS trace.
  - Future gains need either profiling-driven structural work or evaluation on other traces.
