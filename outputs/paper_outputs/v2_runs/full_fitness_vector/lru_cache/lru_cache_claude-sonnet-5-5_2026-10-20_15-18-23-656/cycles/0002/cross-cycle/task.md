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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three hypotheses landed as Winners, and none regressed hit-rate. H01 combined the segmented LRU with the fast hasher, the cheap touch path and removal of the eviction key clone. H02 packed the segment flag into a link field to shrink the segmented node. H03 re-ran the u32 sentinel-link idea, with a hash avalanche fold, on the baseline. H03 is the first test of u32 links, because it was never evaluated in cycle 0001.

## Patterns

- Layout and mechanics changes (u32 links, a lighter hasher, a head-touch early return, a key move) were pure wins that left eviction order alone. Hit-rate came only from the policy, and speed and size came only from mechanics. The axes stayed independent.
- The two changes that shrank metadata were the u32 links in H03 and the segment bit packing in H02. Only H02 paid a per-op-time cost, from mask and accessor work on every link access. H03's size gain came with a speed gain.
- The avalanche fold on the Fx hash appears to have avoided the get p99 regression seen with the weaker hash in cycle 0001. That tail fix has not been confirmed on a segmented variant.
- In H01, put gained most, in both median and tail, so the removed clone and the hasher likely account for most of it. The touch fast path helps only when hits land on the segment head.
- The variant authors could not read the code diffs. Attribution within each bundled change set is inferred, not measured.

## Open questions

- Do u32 links and the segment-bit packing compose? The u32 links change the spare-bit budget the packing relies on.
- Does the hash fold also fix the put p99 and get p99 tails once it is on the segmented policy?
- Which of H01's bundled pieces (hasher, touch fast path, clone removal) carries the speed gain?
- Does the segment-bit masking cost vanish with u32 links, or can the segment be stored implicitly, for example by list membership?

## Continuation

Merge the three branches into one candidate: segmented LRU with u32 sentinel links, the folded Fx hasher, the head-touch early return and the move-based eviction. Pack the segment bit only if its masking cost can be kept off the put path. Next, target the remaining promotion/demotion tail on put, and check hit-rate on workloads other than the scan-heavy trace. Stop re-proving that mechanics changes preserve hit-rate, and stop re-proving that segmentation helps scans.


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
