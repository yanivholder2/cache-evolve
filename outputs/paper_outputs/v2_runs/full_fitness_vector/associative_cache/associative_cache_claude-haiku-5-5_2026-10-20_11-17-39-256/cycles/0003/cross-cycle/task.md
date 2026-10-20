# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: wider buckets via the geometry layout, midpoint promotion of front-half hits, and bucket-local ghost memory for reinsertion. None of the three landed as a Winner. The one with a modest hit-rate gain (wider buckets) paid a small get-path cost, and the other two lost hit-rate on workload-a.

## Patterns

- Changes that move where items sit within a bucket (midpoint promotion, midpoint reinsertion) hurt hit-rate on this workload, and both add shifting work to the hot path.
- Widening buckets was the only change that improved hit-rate, but the gain was small, and the get path paid for the longer linear scan.
- Both bucket-local changes ran into the same suspected limit: their memory windows (back-half protection, the small ghost ring) may be too short to reach the reuse distances in workload-a.
- Put-side and get-side latency moved in ways the diffs do not fully explain in two variants. Put-side measurements should be re-verified before any variant is judged on them.
- Metadata-size did not register any change, so the size probe may be missing the new per-bucket state.

## Open questions

- Does the wider-bucket hit-rate gain hold across other traces, or is it specific to workload-a's conflict pattern?
- Is the ghost ring too small to matter, or is the reinsertion idea itself weak on this trace?
- What is driving the unexplained put and get latency shifts, and are they noise or layout effects?

## Continuation

Next cycle should keep the bucket-width direction and re-measure it before building on it. Drop midpoint promotion as currently specified. Before dropping ghost memory, test it with a longer ghost window or a hit-count gate, and make sure the size probe covers the new state.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes against the frequency-based parent from Cycle 0002: caching the minimum-counter slot for faster victim lookup, merging the victim search with a per-eviction counter halving, and a second-chance reference-bit scheme for victim selection. Only the second-chance variant (v003) landed as a Winner, gaining hit-rate on workload-a and lowering put cost, at the price of slightly higher get cost and larger per-item metadata.

## Patterns

- Both frequency-line variants that tried to speed up the put-side victim search (cached minimum, merged scan with halving) failed to help. Each one either slowed the put path or lost hit-rate, so the put-side cost from Cycle 0002 is not solved by these mechanisms.
- The cached-minimum shortcut was undone by its own maintenance: counter increments on get hits keep invalidating the cached slot, so puts still fall back to the full scan and pay the bookkeeping cost on top.
- Adding a miss-driven decay on top of the existing periodic halving erased the frequency protection too quickly. Aging is already bounded by the periodic schedule, so extra decay is the wrong lever.
- Re-reference-aware victim selection through a second-chance bit retained re-referenced keys through one-shot bursts, the same pollution pattern the frequency work targeted. It gained hit-rate and reduced put cost, which is the first cohort result that improved both at once.
- The Winner's per-item metadata grew, and the get path picked up a small extra write per hit. Metadata growth suggests more state than the single bit the hypothesis specified.
- The Winner's mechanism is inferred, not confirmed: its source overlay was not readable this cycle, so the attribution rests on the hypothesis, fitness vectors, and metadata delta.

## Open questions

- Is the second-chance gain real and mechanism-driven, or does it come from something else in the overlay? The overlay needs to be checked before the attribution is relied on.
- How many bits of per-item state does the Winner actually use, and can the metadata growth be cut back to a single bit without losing the hit-rate gain?
- Does the second-chance gain hold on traces other than workload-a?
- Can the get-side write on every hit be avoided, for example by setting the reference state only on some hits?

## Continuation

Next cycle should build on the second-chance reference scheme: confirm the mechanism, tighten the per-item state toward one bit, and cut the get-side write before adding anything else. Drop the cached-minimum shortcut and the per-eviction halving as axes; neither has a path to a win on the current policy. Keep the periodic halving schedule as is.
</content>


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
