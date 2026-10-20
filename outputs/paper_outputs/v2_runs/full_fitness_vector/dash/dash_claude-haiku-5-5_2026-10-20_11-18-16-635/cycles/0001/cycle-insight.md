# Cycle 0001 Insight

## Cohort outcome

The cohort tried two kinds of change: where promoted entries land in their target bucket (insert position), and how the get path orders its tier lookups (stash scan versus target and probing scans). All three Variants were Winners, but they moved different dimensions, so the Winner label rests on different evidence in each case.

## Patterns

- Insert-position changes act on hit rate. Promoting at MRU protects the promoted key from the next eviction, but it can also displace a hot resident of a full target bucket, so the effect depends on how full the buckets are.
- The get-path reordering is the only change whose stated mechanism is latency. Skipping the stash scan on common target and probing hits is the cohort's clearest per-op improvement.
- Several Variants credit per-op-time movement that their own code paths do not explain, and put-side timing moved in more than one Variant. Treat latency gains from this cohort as unconfirmed until they are re-measured.
- The one Variant with a recorded hit-rate loss (v001) was still a Winner, so the Winner criteria are weighting latency over hit rate. Whether that is intended is unresolved.

## Open questions

- Does the MRU insert in the probing-recovery path lose hit rate on workloads with fuller target buckets, and is the v001 hit-rate loss stable across traces?
- How much of the get-latency gain from reordering the tier lookups is real, and what does stash-hit latency look like once it pays for the two earlier misses?
- Are the unexplained put and per-op movements measurement noise?

## Continuation

Next cycle should pursue the get-path reordering and check that its latency gain holds on stash hits and across reruns. Pursue insert-position work only with a guard against displacing hot target residents, since plain MRU promotion trades hit rate for latency. Drop further claims about put-side or per-op gains until they are re-measured.
