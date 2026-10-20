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
