## Recurring wins

- The second-hit admission filter is the only real policy win. It showed up in cycle 0001, was bit-packed into spare bits at baseline item size in cycle 0002, and was independently re-derived in cycle 0004. Each time it gained hit-rate and kept latency flat or better.
- Packing policy state into spare bits keeps metadata flat. Footprint is no longer a constraint.
- Removing work the policy never reads (dead per-get stats and timestamp writes) helped once, in cycle 0004. The gains from further deletion were too small to measure.
- A lossy doorkeeper cut put-side work sharply (cycle 0006). That is a real lead, but it cost hit-rate.

## Recurring failures

- Hot-path edits to the incumbent have not paid off in five straight cycles (0003 to 0006). These were guards, shortcuts, atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans and reader-writer locks. Each added branches or work and cost latency. The benchmark looks uncontended and the buckets mostly full, so there is little to exploit.
- Insert-position and cold-demotion tweaks overlap with the admission filter instead of adding to it. They trade hit-rate against latency.
- Richer admission or recency signals (counters, ref bits) added no hit-rate beyond the filter. Hit-rate looks saturated on this trace.
- "Winner" status is weak evidence. Flat hit-rate and metadata qualify a variant even when latency regresses.
- Latency moves often appear on paths the change did not touch. The noise floor and code-layout effects have never been measured, so most latency claims are unreliable.
- Variant authors have mostly reasoned from fitness vectors without reading source, so the mechanisms offered are inferred.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for several cycles and is still unanswered.
- Where does get and put time actually go? Nothing has profiled it.
- Does a different trace (scans, frequency skew) change any conclusion? Every result so far comes from the recency-skewed LIRS trace.
- Can a larger or aged doorkeeper keep the put-side savings without losing hit-rate?
- How much of the cycle 0004 re-derivation's gain came from the filter and how much from removing dead bookkeeping? Does it match the cycle 0002 bit-packed incumbent?

## Per-policy notes

- DASH segment promotion (the only policy exercised so far): the bit-packed second-hit filter with dead bookkeeping removed is the base to beat.
- Insert position is exhausted as a lever. Uniform MRU, cold and tiered inserts gave small or noisy effects.
- Admission is saturated beyond the single second-hit gate. A counter or ref bit adds nothing.
- Eviction-order changes (CLOCK-style second chance) gave negligible hit-rate gain and a latency cost.
- Lookup, locking and probing mechanics are not the bottleneck on this workload.
