# Cycle 0005 Insight

## Cohort outcome

All three variants used cycle 0004's v003 as their parent. Each changed one lever:

- **H01** lowered the small-queue floor clamp from `capacity/16` to `capacity/64`.
- **H02** readmitted ghost-hit keys into main with `freq = 1`.
- **H03** moved values into the hash map to get a single-probe `get`.

All three are Winners, but each won something different. v003 ranked first on `metadata-size`, the first real drop in that metric so far, but `get` got slower. v001 is the only variant that improved hit-rate, and only slightly. v002 lost on both hit-rate and latency and made the cut on rank.

## Patterns

- **The lower clamp binds, but only weakly.** Hit-rate moved when the floor was loosened, so the split does reach `capacity/16` on workload-a. The gain was small, which means either the split only sits at the floor part of the time or the benefit of a bigger main levels off quickly. This is the first cycle where the adaptation's bounds turned out to be a live lever. Movement-rule tweaks in earlier cycles did not.
- **Extra credit on readmission hurts.** Admitting a key straight to main from the ghost is already its reward. Adding `freq` on top kept one-time-reuse keys in main for an extra lap, pushed out keys whose reuse was seen through real `get`s, and added rotation work to `evict_main`. Together with cycle 0002's ghosted-main-victims loss, this suggests that changes making the ghost path stronger tend to backfire.
- **The `get` cost is the probe's memory footprint, not the dependent load.** Putting the payload inline widened the hash buckets to a size that is not a power of two, and `get` got slower at both p50 and p99. That is the first `get` move in several cycles that noise does not easily explain. Counting dependent loads predicted the wrong direction.
- **A slimmer `Entry` and a fast `get` pull against each other only when the entry is stored inline in the probe table.** Shrinking the entry inside the slab could reduce `metadata-size` without widening the index.
- **`small_size` is still unreported.** v001 and v002 both added the instrumentation, but neither writer could see its output. So we still cannot tell whether v002's loss came from a split shift or from main residency alone.

## Open questions

- What does `h01_split_report` show: where does `small_size` settle, and how often did the old floor block a shrink? This needs to surface in the variant insights, not just the variant notes.
- Is the grow-small trigger, which pulls the split back up, now the stronger lever than the floor?
- Is v001's `get` regression noise? Its `get` path is byte-identical to the parent's.
- Can the slab keep a key-free 16-byte `Entry` while the index stays at the parent's narrow `(K, u32)` buckets?
- Would conditional readmit credit (only on a second ghost hit, or only while small is at its floor) behave differently from the blanket version?

## Continuation

Use v001 (the `capacity/64` floor) as the policy parent, since it is the only hit-rate gain. Don't lower the floor any further. Probe the grow-small trigger strength or the upper clamp next, one lever at a time with ghost size pinned. Drop blanket readmit-`freq` credit and inline-payload index layouts. To shrink `metadata-size`, try a key-free `Entry` that stays in the slab behind the narrow index. Each variant must get its settled `small_size` figure into its insight, and must not credit `get` changes unless the `get` path or the index layout changed.
