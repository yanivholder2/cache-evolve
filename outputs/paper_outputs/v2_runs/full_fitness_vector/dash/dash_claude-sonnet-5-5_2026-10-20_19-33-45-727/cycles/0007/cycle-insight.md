# Cycle 0007 Insight

## Cohort outcome

Three variants on the doorkeeper-style admission incumbent: a larger two-hash aged doorkeeper (v001), a one-slot ghost fingerprint per segment to re-admit the last evicted key (v002), and a deletion-only cleanup of dead writes and reads (v003). All were Winners, but none improved latency. Only v001 raised hit-rate, and v002 and v003 were flat on hit-rate.

## Patterns

- Per-op-time got worse in all three variants, including get latency where get was not meant to change. This is the sixth cycle in a row in which a hot-path or admission edit regressed latency, and the pattern points to layout effects or noise more than real cost.
- v001 confirms the cycle 0006 trade-off. A bigger, aged doorkeeper recovers the lost hit-rate, but it costs put latency through extra probes, cache footprint and periodic resets.
- v002's one-slot ghost is almost free in metadata but almost never matches a reusable key, so it added work for no real hit-rate gain.
- v003 shows that deleting more dead bookkeeping does not help. The cycle 0004 gain most likely came from the filter itself and not from dead-code removal.
- Winner status is again weak evidence, because flat hit-rate and metadata are enough to qualify.
- Variant authors again could not read the source, so the mechanisms above are inferred from fitness vectors.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for four cycles and still has no answer.
- Can a doorkeeper keep v001's hit-rate while staying cheap, for example by fitting in a single cache line?
- Would a larger or better targeted ghost history gain hit-rate, or is it not worth its cost on this trace?

## Continuation

Stop making latency-motivated code edits and drop further dead-store deletion, one-slot ghost and generic doorkeeper-size variants. The next cycle must first measure the noise floor by repeating baseline runs, and profile or use a second trace, before crediting any latency change. If a policy variant is tried, make it a cache-line-sized aged doorkeeper that holds v001's hit-rate at lower put cost.
