# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes around the full-scan, lowest-counter victim choice: counting overwrites of resident keys as reuse (v001), a per-slot tag used as a probe-time filter (v002), and breaking counter ties by residency class instead of age (v003). None produced a clear helped dimension. Only v003 moved hit rate, and only marginally on workload-a. All three record slower get latency, and all three are labelled Winners anyway.

## Patterns

- Get latency regressed in all three variants. v002's get slowdown has a plausible path (the tag read and branch per probe), but v001 and v003 both touched paths their hypotheses did not describe as get-side. A shared cause, such as overlay placement or measurement variance, is more likely than three independent mechanisms.
- Put latency rose in all three. v001's rise is expected from its added increment. v002's and v003's are attributed to bookkeeping and an extra comparison on the eviction scan. Together with cycles 0006 to 0009, this reinforces that put cost is not explained by the victim-scan or per-write logic the hypotheses name.
- Hit rate is largely insensitive to the signals tried. Counting overwrites (v001) and the tag filter (v002, which lowered hit rate) did not help. The residency tie-break (v003) is at most a weak lever. Retention still appears to come from hits accumulated after insertion, consistent with cycle 0009.
- Keys are u64 in the packed record, so any change that saves a full key compare (v002) recovers little work. Per-slot filters only pay if key comparison is expensive here.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 is still outstanding.
- The overlay source was not inspected in any variant, so every mechanism above is inferred from the hypotheses, not confirmed against the diff.

## Open questions

- Is the get-path slowdown common to all three variants, and if so, is it an overlay placement or measurement effect rather than a property of any one change?
- Does the residency tie-break (v003) hold its marginal hit-rate gain on a second trace?
- Why did the per-slot tag (v002) lower hit rate when it was meant to leave eviction and promotion untouched? Was the tag mis-maintained on promotion, or did it turn real hits into misses?
- Why are all three variants Winners when none records a clearly helped dimension? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the victim-selection base and stop spending hypotheses on overwrite counting (v001), per-slot tag filters over u64 keys (v002), or residency-class tie-breaks (v003) unless the second-trace check shows a gain. Before any further variant is judged on get or put latency, re-measure the parent to determine whether the get slowdown is shared across this cohort. Do not credit any per-op or hit-rate result from this cohort until the second-trace check has been run.
