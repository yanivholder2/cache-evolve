# Cycle 0002 Insight

## Cohort outcome

The cohort tried a fingerprint prefilter on bucket lookups, a longer bucket-local ghost ring, and frequency-based victim selection using per-slot access counters. Only the frequency-based victim selection (v003) landed as a Winner, and it gained hit-rate on workload-a at a cost in per-op time.

## Patterns

- Frequency signals helped hit-rate where recency-only ordering had not. Re-referenced keys survived bursts of one-shot inserts, which is the pollution pattern the bucket-local work has been chasing.
- The gain came with a victim search that scans the full bucket on put, so the cost sits on the hot path. The per-op regression is the main thing the Winner has to fix.
- Lengthening the ghost ring did not lift hit-rate. This closes the question cycle 0001 left open about whether the ring was simply too short, and points to the reinsertion idea itself as the weak part.
- Adding a fingerprint prefilter to 32-slot buckets added a second linear scan on get and extra write work on every put. Hit-rate was unchanged, so the regression is isolated to the layer itself.
- Metadata-size reported no change in any variant, including ones that clearly add per-bucket or per-slot state. The size probe does not appear to count the new state, so it should not be used to judge storage cost this cycle.
- Get and put latency movements were single-run and in some cases not explained by the diffs. Treat the size of any timing gain as uncertain until re-measured.

## Open questions

- Can the frequency victim search be made cheaper (tracking a running minimum, or fewer ways) without losing the hit-rate gain?
- Does the frequency gain hold on traces other than workload-a, or is it tied to its one-shot insert pattern?
- What does the size probe need to count so that per-bucket and per-slot state shows up in metadata-size?

## Continuation

Next cycle should build on frequency-based victim selection and attack its put-side cost first, keeping hit-rate fixed. Drop the fingerprint prefilter and ghost-ring length as axes. Do not tune ghost-ring length further.
