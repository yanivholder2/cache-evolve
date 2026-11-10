# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort had three single-purpose tests: v001 merged cycle-3's two filters (reuse-gated main-victim ghosting plus freq/reused reset on graduation), v002 raised the ghost bound from `main_size` to the full capacity, and v003 merged the resident and ghost indexes into one map. Only v001 was a real win. It improved hit rate at every trace/size point, with entry size unchanged and no latency cost. v002 regressed web_0 and large hm_0, and v003 kept hit rates bit-identical but got slower. Both count as Winners only by rank. Neither should be built on.

## Patterns

- The filters add up. Gating ghosts on reuse and resetting credit at graduation stack cleanly once both judge a key only by its reuse inside main. hm_0 gains more than web_0, which matches every cycle so far: hm_0 responds to how main retains keys.
- More ghost history is harmful, not neutral. A longer window lets stale reuse back into main, where it pushes out live residents. Cycles 3 and 4 together point one way: ghost budget should be spent more selectively, not made larger. This rules out the ghost bound as the cause of web_0's remaining large-size gap.
- Probe count is not the bottleneck for timing. Halving probes lost to higher load factor and wider buckets, which hurt the tails most. Table occupancy and bucket width matter more than the number of lookups. Structural timing work now needs to budget for memory layout as well as the algorithm.
- Fixed-struct size keeps shifting (v002 up, v003 down) while per-entry metadata stays flat across the cohort. The summary metric still misses live ghost memory.

## Open questions

- Is the small-queue share (`capacity / 4`) what's left behind web_0's large-size loss? It is the last remaining candidate.
- Now that the merge works, do the graduation threshold or `MAX_FREQ` give more gain on hm_0 without costing web_0?
- Would a unified index pay off with generous headroom or a compact slot encoding (a `u32` slab index plus a ghost flag bit), or is the two-table layout already close to optimal?

## Continuation

Use v001 (Fx slab + both filters) as the only parent. Keep the ghost bound at `main_size` or lower, and drop the idea of a longer ghost. For web_0, test the small-queue share one variable at a time. For hm_0, try main-retention knobs (`MAX_FREQ`, graduation threshold) as separate variants. If you retry index merging, pair it with explicit load-factor headroom or a compact slot. Otherwise, drop structural probe-count work. As before, give no timing credit to policy-only diffs.


### Cycle 0005

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002 (MAX_FREQ 7 on the two-filter parent). Each tested one way to fix the cycle's remaining gaps: v001 grew the small queue to 1/3, v002 gave freq-1 small keys one reinsertion into small, and v003 re-admitted main-victim ghost hits with `freq = 1`. None clearly beat the parent. v002 and v003 each gained a sliver in one place and lost a sliver in another, and v001 lost slightly everywhere. All three are Winners by rank only, and none should replace the parent.

## Patterns

- web_0 at the large size slipped in every variant, the cell two of the three hypotheses targeted. Cycle 5's reading that this gap is about probation length is now falsified from both sides. Shrinking small (cycle 5) and growing it (v001) both regressed, so the 1/4 share sits at a local optimum. Extending probation only for freq-1 keys (v002) also failed to help.
- The ghost queue already acts as the second chance for demoted keys. v002's in-small reinsertion mostly duplicated the ghost→main path and pushed out newcomers. v003's extra credit on the ghost→main path helped hm_0 but let stale returners displace live residents on web_0. Whatever web_0 lacks sits in how the ghost hands keys to main, not in what happens inside small.
- hm_0 at the small size is still stuck. Mechanisms that fire only on ghost hits are too rare at 16384 to move it, because the ghost there is short and reuse-gated. v003's gain showed up at the large size instead, the opposite of what it predicted.
- Timing gave the same lesson again. Policy-only diffs showed noise-level shifts. The one structural cost, v002's multi-pop eviction loop, clearly inflated the `put` tail: any eviction path whose work per miss is unbounded is a tail-latency risk.
- Per-entry metadata stayed flat. Both new bits fit into existing padding or tag space.

## Open questions

- Would a stricter ghost→main admission rule for small-demotee ghost hits (for example re-admitting to small instead of main) close web_0's large-size gap? That would be the opposite direction from v003.
- Does the `freq > 1` threshold, or a main-eviction lever that acts on every sweep, move hm_0 at 16384 where ghost-path changes cannot?
- Could v003's large-size hm_0 gain be kept without the web_0 cost, for example by applying the credit only when the ghost hit comes soon after eviction?

## Continuation

Keep cycle-5 v002 as the parent. No cycle-6 variant replaces it. Drop small-queue resizing and any extra rounds inside small, since probation length is ruled out. For web_0, test the ghost→main admission rule as a single variable, preferring stricter admission over looser. For small-cache hm_0, try levers that act on every main eviction, not on ghost hits. The ghost-index shrink for the `put` tail is still untested. Keep eviction work bounded per miss, and give no timing credit to policy-only diffs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
