## Recurring wins

- Admission filtering that keeps one-hit keys out of the hot region is the only lever that has repeatedly paid off (second-hit filter, then doorkeeper variants). It also saves put-side insert and eviction work.
- Packing filter state into spare bits keeps item size at baseline, so footprint is not a constraint.
- Subtractive re-derivation from baseline helped once (dropping unread stats), by removing work the policy never reads.
- A doorkeeper held to one cache line and aged often enough to stay sparse gave the best trade-off so far. It matched a large filter's hit-rate at lower put cost.

## Recurring failures

- Insert-position tweaks, cold demotion and cold first admission overlap with the admission filter. They do not stack, and they trade hit-rate against latency.
- Richer signals (multi-bit counters, ref bits, ghost entries) added hot-path work for negligible hit-rate gain. The filter already catches most one-touch items.
- Blind hot-path edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scan, RW-lock upgrade) made latency slightly worse across many cycles. Probing, locking and hashing are not where time goes, and the benchmark looks uncontended.
- Further dead-store deletion stopped helping once the main dead bookkeeping was gone.
- Small unaged or lossy filters cut latency but lose hit-rate.
- Warm-up admission into free slots gave nothing, because the cache is almost always in the full-segment regime.
- "Winner" status is weak evidence, since flat hit-rate and metadata qualify a variant even when latency regresses. Many latency regressions appeared on untouched paths, which points to layout or measurement noise.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for five cycles and is still unmeasured, and every small latency claim depends on it.
- Where do get and put time actually go? Nobody has profiled this.
- Do the conclusions hold on a second trace with scans or frequency skew? All results so far come from the recency-skewed LIRS trace.
- Is the one-line doorkeeper's small hit-rate and get gain real or noise?
- Would a different aging threshold or probe count, or an aged fingerprint filter, push the hit-rate and put-cost trade-off further?

## Per-policy notes

- DASH segment promotion and admission: policy levers on insert position and eviction order look saturated on the LIRS trace. Admission is the remaining live lever.
- Second-hit filter (bit-packed, baseline item size): the long-standing base. It was later replaced as the incumbent by the doorkeeper family.
- Doorkeeper ring or table: a size and aging trade-off. Too small or unaged loses hit-rate, too large costs put latency, and one cache line with frequent aging is the current best (adopted as incumbent). Next steps are tuning aging and probe count only, with no edits to get.
- Ghost or history entries: a one-slot ghost almost never matches a reusable key, so it was not worth its cost.
