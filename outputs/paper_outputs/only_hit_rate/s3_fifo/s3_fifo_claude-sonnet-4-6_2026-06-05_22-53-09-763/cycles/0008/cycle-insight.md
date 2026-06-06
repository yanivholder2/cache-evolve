# Cycle 0008 Insight

## Cohort outcome

This cycle revisited three axes — second-chance threshold tightening, small-queue size reduction, and unconditional move-to-front on main hits — yielding one regression, one clear winner, and one narrow positive. The winner (v002) reopens the queue-sizing dimension that had been considered settled.

## Patterns

- **Cross-feature interactions can invalidate previously successful axes.** Raising the second-chance threshold (v001) was a profitable direction in isolation, but eager promotion introduced in cycle 0007 now deposits items into main already carrying freq==2. The tighter gate evicts those freshly-promoted hot items on their first pass, turning what was a "stale squatter filter" into an accidental hot-item ejector. Threshold adjustments that worked before a structural change may not transfer across it.
- **The small-queue sizing benefit is monotone past the 10% floor.** Shrinking to ~7% continued to improve resident quality by transferring slots to main; the prior characterisation of 10% as near-optimal was an untested assumption rather than an empirical bound. This axis is not yet exhausted.
- **Unconditional MTF on main hits delivers a real but narrow gain.** Proactive recency refresh prevents drift to the eviction tail, but because evict_main()'s reactive recycling already rescues most hot items on Zipfian workloads, the marginal value is incremental rather than structural.

## Open questions

- Does the small-queue benefit continue below ~7%, or is there a floor where the filter becomes too coarse and cold items begin polluting main in meaningful numbers?
- Given that eager promotion sets the entry frequency for main-queue arrivals, is there a second-chance threshold formulation — perhaps frequency-relative rather than absolute — that captures the original intent without penalising recently-promoted items?

## Continuation

The most actionable finding is that small-queue sizing retains room to run: the next cycle should probe at least one further reduction below ~7% to locate the true floor. The second-chance threshold axis is not closed, but any revisit must reason explicitly about the freq==2 entry distribution created by eager promotion — a naive increment repeats v001's failure. The unconditional MTF result (v003) is a confirmed small positive that can serve as a base feature; it does not need further dedicated probing on this workload.
