# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three variants refined the segmented LRU: u32 links on the fast segmented cache (v001), segmentation built onto the u32-link plain LRU with the segment bit packed into `next` (v002), and lazy demotion moved from `get` to `put` (v003). All three were Winners, but each paid for it in per-op-time. Nothing improved speed, and the only gains were metadata size (v001) and hit-rate (v002).

## Patterns

- Every variant slowed per-op-time, including tails. This contrasts with cycle 0002, where u32 links and the hasher were pure wins on the plain LRU. Once the policy is segmented, link-width and bit-packing changes no longer bring speed.
- Narrowing links shrank the cache (v001), but the added widening and NIL checks, plus the hash fold, cost latency. Locality did not pay off at this workload size. The saving is size only, and it is not confirmed that u32 links on a segmented policy are a speed win.
- v002 again showed that the cost of segmentation is the promotion/demotion relinking on the hot path. Packing the flag keeps per-node size flat but cannot remove that work. The container grows from the extra head/tail pair, and a per-node metric hides this.
- v003 showed that deferring demotion only moves the relink work. It landed on `put`, the more latency-sensitive path, and `get` did not improve. Hit-rate was preserved.
- The remaining time cost of segmentation is the promotion/demotion pointer surgery itself, and reshuffling it between operations does not fix that.
- Variant authors could not read the diffs. Attribution among the hash fold, index conversion and flag masking is inferred.

## Open questions

- Which piece of v001 caused the slowdown: the hash fold or the u32 conversions? The fold was bundled with the layout change.
- Can promotion be made cheaper without eliminating it, for example by promoting only when a node is far from the head, or by promoting on a sampled or deferred basis?
- Can the segment be stored implicitly, such as by list membership, so that no masking is needed on link access?
- Does segmentation's hit-rate gain justify its latency cost on workloads other than the scan-heavy trace?

## Continuation

Stop shifting demotion work between `get` and `put`, and stop bundling the hash fold with layout changes. Next, run an isolated test of the fold against the u32 conversion cost. Then attack promotion cost directly: skip the relink for nodes already near the head, or batch or amortize promotion. Treat the cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) as the baseline, and judge new variants by whether they recover speed without losing hit-rate.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
