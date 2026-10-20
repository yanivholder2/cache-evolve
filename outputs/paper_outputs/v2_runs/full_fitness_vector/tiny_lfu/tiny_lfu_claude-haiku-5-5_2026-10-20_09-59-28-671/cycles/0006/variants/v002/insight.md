# Variant v002 Insight

## Helped

- per-op-time: Get and put latencies at the tail (p99) and the get median came in below the parent, even though H02 targeted hit-rate and did not mention timing. The demotion branch is probably rarely taken on this trace, so the likely source is incidental layout or branching change rather than the demotion work itself; this is a single run, so treat the gain as provisional.

## Hurt

- hit-rate: No movement at all against the parent on workload-a at size 2048. H02 was accepted only on a strict improvement, so the protected-tail demotion did not change any eviction outcome on this trace. The likely reasons are that the protected segment never reached its capacity limit during the run, or that the demoted entries were not re-referenced before eviction.

## Takeaway

Demoting the protected tail to probation MRU did not change hit-rate on workload-a, so the segmentation gain in the lineage does not appear to depend on protected-segment turnover at this size. Before another attempt at turnover, check whether the protected-capacity branch fires on this trace at all; if it never fires, the hypothesis was untestable here rather than refuted. Note: the changed source could not be inspected in this review, so this conclusion rests on the fitness vectors and hypothesis text alone.
