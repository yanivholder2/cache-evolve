# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three hypotheses were tried against the baseline: compact u32 arena links (H01), a segmented LRU (H02), and a faster hasher with cheaper touch/eviction paths (H03). The Winners were the segmented LRU (v002), which gained hit-rate, and the Fx-hasher/touch variant (v003), which gained median latency and a smaller struct. The compact-links variant (v001) was rejected before evaluation because the simulator build failed on a Maven download rate limit (HTTP 429). That is an infrastructure failure, so H01 is untested.

## Patterns

- The two Winners improved different axes: v002 improved hit-rate through scan resistance, and v003 improved per-op-time and struct size with no hit-rate change. They are largely independent and could be combined.
- Both Winners paid for their gains in tail latency. Promotion/demotion list surgery hurt put p99 in v002, and weak hashing hurt get p99 in v003. Median gains came with worse tails in this cohort.
- v002 also increased metadata (a larger node and extra list bookkeeping). v003 reduced it. Metadata-size is a real axis that policy changes tend to worsen.
- v003 only partly landed H03: the key clone on the eviction path in `put` remained, so some of the available put savings were left unused.

## Open questions

- Can the hash be made fast without the tail regression, for example with a better mixing or fold step?
- Can segmentation's metadata cost be recovered, for example by encoding the segment bit in existing fields or using compact links?
- Does the v002 hit-rate gain hold on traces other than the scan-heavy lirs workload-a?
- Does H01's u32 links idea deliver its locality and size benefit once it is evaluated?

## Continuation

Re-attempt H01 (u32 sentinel links), since v001 failed for infrastructure reasons and not on merit. Build on the Winners: combine segmentation with the fast-path touch and a better-mixed hasher, and finish removing the eviction-path key clone. Prioritize fixing the tail regressions (get p99 from hashing, put p99 from promotion/demotion). Do not spend more effort on proving that segmentation helps scan workloads; that is established.


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
