# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted node memory compaction or hit-rate improvement via structural redesign; only v002's sentinel-based link compression landed as a clean winner, delivering smaller nodes and better get latency without sacrificing eviction quality.

## Patterns

- **Metadata compaction was the most reliable lever.** Both v002 (u32 sentinel links) and v003 (CLOCK flat slots) halved node size — but compaction that preserved the LRU chain (v002) also improved get latency through better cache-line utilization, while compaction that eliminated the chain (v003) introduced a new bottleneck via the CLOCK hand's sweep-induced cache-line pollution.
- **Algorithm complexity adds hidden per-op cost.** The two-queue variant (v001) and CLOCK (v003) both introduced extra pointer work or array traversal that eroded or negated their structural gains. The simpler structural change (v002) outperformed both.
- **Exact LRU ordering remains valuable for this workload.** v001's two-queue policy improved hit rate by protecting hot items, but v003's coarser single-bit recency signal hurt hit rate — suggesting the workload has enough reuse that precision in eviction decisions pays off.
- **Put latency is a secondary but real concern.** v002's slight put regression from cast overhead and v003's put blowup from CLOCK sweeps both appeared; get-path improvements should not be assumed to carry over to put.

## Open questions

- Can the two-queue hit-rate gain from v001 be recovered on top of v002's leaner node layout, where the extra per-queue bookkeeping fields represent a smaller fractional overhead?
- Is CLOCK's cache-line pollution inherent to the algorithm, or does a segmented or lazy-sweep variant recover the structural win without the latency penalty?
- How much further can node size be reduced (e.g., toward a single cache line per node) before diminishing returns set in?

## Continuation

v002's u32 sentinel-based links are the new structural baseline — next cycle should treat this as the parent and compound on it. The most promising directions are: (1) revisiting the two-queue policy on top of v002's leaner nodes to see whether hit-rate gains now come at an acceptable latency cost, and (2) exploring further struct packing to push node density higher. CLOCK-style approaches should be deprioritized unless a mechanism to avoid hand-sweep cache pollution is proposed.


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
