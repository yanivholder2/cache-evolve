# Variant v003 Insight

## Helped

(No dimension improved. Hit-rate matches v002 exactly in every trace and size cell, including web_0 at 131072, the cell this change was meant to fix. Metadata-size did not change, as intended, because the reuse flag reuses the existing `Item::lfu_counter` field.)

## Hurt

- per-op-time: `get` and `put` got slower at both p50 and p99, and gave back v002's latency advantage. The code adds very little: a conditional store of `lfu_counter` on target and probing hits, a clear on stash entry, and a flag check plus clear in `place_in_other_or_stash`. More cold evictees now go to stash instead of taking a free slot, which brings back some stash writes and later stash-hit work that v002 avoided. Still, a regression this large is hard to explain from such small changes. Together with v002's own caution that its speed-up may be partly noise, this suggests the per-op-time benchmark is not stable enough to separate these two variants.

## Takeaway

Limiting v002's free-slot moves to items hit while in the normal tier left every hit-rate cell unchanged. This undercuts v002's explanation that cold evictees taking free probing slots caused its web_0/131072 loss. That free-slot placement policy is probably not what limits hit-rate at either size. The latency question is also still open: v002's latency gain was not preserved, and the p50/p99 numbers should be re-measured before any latency conclusion is drawn from v002 or v003.
