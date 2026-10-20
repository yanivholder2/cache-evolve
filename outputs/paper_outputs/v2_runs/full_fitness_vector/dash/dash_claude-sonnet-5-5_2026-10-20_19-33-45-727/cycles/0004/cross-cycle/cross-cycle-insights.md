## Recurring wins

- The second-hit admission filter is the one real, repeatedly confirmed win (cycles 0001, 0002, 0004). It screens one-hit-wonders, skips first-hit remove-and-promote work, and was independently re-derived from baseline with the same benefit.
- Packing the filter flag into spare bits keeps metadata at baseline item size while keeping the hit-rate and latency benefit (cycle 0002).
- Subtractive changes pay off: removing bookkeeping the policy never reads (per-get stats, timestamp writes) improved latency (cycle 0004).

## Recurring failures

- Position-only levers (uniform MRU, cold/hot insert, cold demotion) gave small, mixed or noise-level gains. Stacking them on the filter overlaps, because they target the same scan resistance, and trades hit-rate against latency.
- Richer admission or recency signals (multi-bit counter, ref bit) added almost no hit-rate on top of the filter and cost latency. Hit-rate looks saturated on this trace.
- Adding hot-path mechanisms (guards, shortcuts, atomics, early returns, free-slot fast paths, merged scans) consistently lost latency. Branches that only sometimes succeed are not free, and buckets are mostly full.
- "Winner" status is weak evidence: many gains are within trace or run noise, and regressions sometimes appeared on untouched paths (layout or measurement noise).

## Open questions

- Which latency deltas are real versus run-to-run or code-layout noise? Re-measure with repeats.
- Where do get and put time actually go (lookup, locking, bucket scan)? Nothing has measured this.
- How much of the cycle 0004 v003 gain comes from dropping dead bookkeeping versus the filter itself?
- Does the cleaned-up minimal filter match or beat the cycle 0002 v001 incumbent?
- Do conclusions hold on a different trace (scans, frequency skew) rather than the recency-skewed LIRS trace?

## Per-policy notes

- DASH segment with the bit-packed second-hit filter: current best base. Use the cycle 0004 v003 minimal version, compared against cycle 0002 v001.
- Promotion and insert-position variants (MRU, tiered, cold first admission, cold demotion): considered exhausted. Do not revisit without a latency re-measurement.
- Eviction-order variants (CLOCK-style second chance): no hit-rate benefit, extra scan cost.
- Next direction: subtractive latency work and a different evaluation trace, not new policy levers.
