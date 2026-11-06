# Variant v001 Insight

## Helped

- per-op-time (`put` p50, p99): Both put percentiles fell back to roughly the level before the mask existed. Every stash removal now does one hash and one subtraction on the removed key instead of a `refresh_stash_mask` re-hash of the whole bucket. This confirms that recomputing the mask was the entire cause of 0007 v002's put regression. The extra `stash_masks` memory touch on insertion costs very little by comparison.
- per-op-time (`get` p99): The get tail recovered most of what v002 lost, because a stash hit that promotes out of stash no longer pays for a bucket-wide recompute. It still sits above the tail from before the mask, which was the hypothesis's target. Some tail cost therefore remains: false-positive passes that pay for the mask test plus the full stash scan.

## Hurt

- per-op-time (`get` p50, the summary metric): The median get lost all of v002's gain and ended up slightly worse than the median from before the mask. Shrinking the filter from 64 one-bit slots to 16 four-bit counters raises the chance that a normal-tier hit finds a non-zero counter for a key that is not in stash. That hit then pays for the extra `stash_masks` load and the `get_position` scan anyway. Once false positives are common enough, the filter costs more than having no filter. A secondary possibility I could not check without the source: debug-only cross-check code (`#[cfg(debug_assertions)]` recompute or asserts) might still be live in the benchmarked build.

## Takeaway

Counting-mask maintenance fixes the write-side cost completely, so upkeep was never the problem. The 16-counter filter is too coarse to keep the read-median benefit, though. In this design the p50 lever is the filter's false-positive rate, not how cheap its update is. The next step is to get O(1) deletion without giving up resolution. Options: wider counter words (for example a u128 or 2×u64 holding 32+ counters, or 2-bit counters, since stash buckets hold only a few slots), or using a second hash bit to cut false positives, while keeping the add/subtract update path.

(Note: I could not list or open the overlay source in this session, so this analysis rests on the hypothesis's specified changes and the parent/child fitness vectors, not on a line-by-line diff.)
