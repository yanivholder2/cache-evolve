# Cycle 0002 Insight

## Cohort outcome

Three single-lever changes were tried: sketch aging on top of the windowed design (v001), a larger window (v002), and a doorkeeper in front of the baseline sketch (v003). None improved hit-rate over its parent. v002 was named Winner, but its own insight reports a small hit-rate loss. It looks like the least-bad variant, not a real gain.

## Patterns

- Every variant moved hit-rate slightly the wrong way on workload-a at the tested cache size. The workload looks stable and frequency-driven, with little recency or popularity drift for these mechanisms to exploit.
- Each change that discards or hides frequency information cost hit-rate. Aging decays history, a larger window dilutes main-region protection, and the doorkeeper hides first sightings from the counters.
- Metadata-size and per-op-time are still missing from the fitness vectors. This is the second cycle with that gap, so the cost side of every idea is still unmeasured.
- Variant authors could not always inspect overlay source, so the explanations are plausible readings and not confirmed causes.

## Open questions

- Is the Winner status of v002 an artifact of ranking among losers, or is it a real gain on some dimension that wasn't reported?
- Would aging pay off with a longer sample period, or on a trace with real popularity shifts?
- Would a doorkeeper help if admission were adjusted for its +1 offset, or on a workload with real sketch pollution?
- Do the cycle-1 and cycle-2 changes differ in metadata or per-op cost even though hit-rate barely moves?

## Continuation

Stop tuning window size and stop adding decay or filtering on workload-a alone, since both lost hit-rate. First fix the fitness output so it includes metadata-size and per-op-time, and add a second trace with shifting popularity so aging and doorkeeper ideas can be judged fairly. Then pursue cost-oriented changes, such as v003-from-cycle-1's compact sketch, or admission tuning that preserves frequency information, and judge them on cost as well as hit-rate.
