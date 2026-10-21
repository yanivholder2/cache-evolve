## Recurring wins

- The second-hit admission filter is the one real policy win. It was found in cycle 0001, bit-packed to baseline item size in cycle 0002, and independently re-derived from baseline in cycle 0004, where it improved both hit-rate and latency.
- Packing flags into spare bits keeps metadata flat. Footprint is no longer a constraint for counters or ref bits.
- Scan resistance (keeping one-touch items out of the hot tiers) is the lever that works. Admission gating beats insert-position tuning.
- A doorkeeper that declines first touches saves put-side work (insert and eviction). This is the only credible latency mechanism seen, but it costs hit-rate when the ring is small.

## Recurring failures

- Insert-position-only variants and stacked position/admission rules overlap and do not add. They gave small or mixed gains and are exhausted.
- Richer admission or recency signals (multi-bit counter, ref bit, one-slot ghost) added almost no hit-rate. The second-hit filter already catches most of what they target.
- Hot-path edits to the incumbent regressed latency in roughly six consecutive cycles. These included guards, shortcuts, lookup atomics, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks and dead-store deletion. Extra branches or work cost more than they save.
- Regressions often appeared on paths the change did not touch, so layout effects or noise are likely. The noise floor has never been measured.
- Subtractive cleanup ran out of room. The cycle 0004 gain probably came from the filter, not from dead-code removal.
- Winner status is weak evidence. Flat hit-rate and metadata qualify a variant even when latency regresses.
- Variant authors could not read the source, so the mechanisms offered are inferred.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for four or more cycles and is still unanswered.
- Where does get and put time actually go (lock, memory access, bucket scan)? Nothing has profiled it.
- Does a second trace (scans, frequency skew) change any conclusion? Everything so far uses the recency-skewed LIRS trace.
- Can a cache-line-sized, aged doorkeeper keep the larger doorkeeper's hit-rate and the small ring's put savings?
- Does the cleaned-up minimal filter match or beat the cycle 0002 bit-packed incumbent?

## Per-policy notes

- DASH second-hit bit-packed filter: this is the incumbent and base. It has baseline item size, and hit-rate looks saturated on this trace.
- DASH doorkeeper ring (small fixed): large put-side latency drop but lost hit-rate because recurring keys age out before their second touch.
- DASH doorkeeper (larger, two-hash, aged): recovers hit-rate but costs put latency through extra probes, footprint and resets.
- DASH position variants (uniform MRU, cold/hot split, cold demotion): modest early hit-rate gains, with trade-offs against latency. Drop them.
- Ghost history (one slot): near-free in metadata but almost never matches. A larger ghost is untested and of doubtful value.
- Next step: measure the noise floor and profile, or use a second trace, before crediting any latency change.
