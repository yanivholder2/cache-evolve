# Cycle 0005 Insight

## Cohort outcome

All three variants were single-change children of cycle-4 v001 (Fx slab + both filters). v001 shrank the small queue from 25% to 10%, v002 raised `MAX_FREQ` from 3 to 7, and v003 pre-sized both hash indexes to twice their occupancy. Only v002 gained, and only slightly in summary, mostly on hm_0 at the small size. v001 regressed web_0 at both sizes and lost on the summary. v003 kept hit rates bit-identical but made tails worse. All three count as Winners by rank only. v002 is the only one worth carrying forward.

## Patterns

- Every gain this cycle sits at the small cache size, and mostly on hm_0. A bigger main (v001) and a deeper freq cap (v002) both helped hm_0 at 16384 and did nothing or hurt at 131072. Main retention is close to saturated at the large size. Whatever is left on hm_0 is a small-cache problem.
- web_0's large-size gap is a probation-window problem. Shrinking the small queue widened the gap, so web_0 needs new keys to stay in probation long enough to get the second hit that graduation requires. All structural candidates (ghost bound, small-queue share) have now been tested. What remains is to make probation longer or graduation easier, not to resize main.
- The layout lesson from cycle 4 is about footprint, not load factor. More headroom hurt tails just as the denser merged index did. hashbrown's rounding already leaves the resident index about half empty. Memory footprint and locality drive the `put` tail, so the only layout direction left is shrinking tables, especially the ghost index.
- `put` p99 moved slightly up in every variant: longer ghost compactions (v001), longer second-chance sweeps (v002), sparser tables (v003). Only v003's regression is clearly caused by its diff. The other two shifts are within noise.
- Per-entry metadata stayed flat again, while v003 doubled index heap without the metric showing it.

## Open questions

- Does a larger small queue (for example 1/3), or a softer graduation threshold applied only inside probation, close web_0's large-size gap without bringing back cycle-2's `freq >= 1` regression?
- Would a size-aware `MAX_FREQ`, or a split between small and main that depends on cache size, keep v002's small-cache hm_0 gain without its large-size flatness?
- Can the ghost index be made denser (for example by storing fingerprints or compact keys) to cut `put` tail cost?

## Continuation

Use v002 (MAX_FREQ 7 on the two-filter parent) as the new parent. Raising the cap further is unlikely to pay off. Drop small-queue shrinking and index over-allocation. For web_0, test one variable at a time: a longer probation window (small share above 25%) or easier graduation inside probation. For implementation, only shrink tables, with the ghost index first. As before, give no timing credit to policy-only diffs.
