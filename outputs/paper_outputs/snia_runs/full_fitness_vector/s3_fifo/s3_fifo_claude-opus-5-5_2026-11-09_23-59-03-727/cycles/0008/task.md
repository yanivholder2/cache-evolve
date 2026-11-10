# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant made one targeted change. v001 used stricter ghost→main admission: small-demotee ghost hits went back into small with `freq = 1`. v002 halved main freq on each sweep instead of decrementing it, built on cycle-6 v003. v003 replaced the ghost deque with a fixed `main_size` ring. None beat the parent (cycle-5 v002). v001 and v002 lost summary hit rate, v003 kept hit rate flat but made the `put` tail worse, and v001 also inflated `put` latency. All three are Winners by rank only.

## Patterns

- The ghost handoff is ruled out as the lever for web_0 at 131072. Looser admission (cycle-6 v003) and stricter admission (v001) both regressed that cell. On this parent, a small-demotee ghost hit already confirms reuse. Forcing another probation pass sends long-reuse-distance keys round the ghost → small → ghost loop and takes small slots from new arrivals. v001's only gain came at web_0 16384, where main turns over fast.
- Faster aging in main hurts. Halving evicts high-freq keys whose inter-hit gap is longer than a sweep, and hm_0 at 16384, the cell it targeted, fell. Together with cycle 5's MAX_FREQ gain, this suggests strongly reused keys need more tenure in main, not less. The "dead residency" theory did not hold up.
- The `put` tail is set by hash probes per miss, not by occasional O(n) cleanup. The ring's constant per-insert liveness probe cost more than the parent's rare, amortized `retain`/scan. v001's extra `evict_small` cascade and ghost churn also raised the tail. Changing the shape of the ghost container is not the lever.
- Per-entry metadata stayed flat again. All diffs fit inside the existing 24-byte entry.

## Open questions

- After probation length, small share, ghost bound and both directions of ghost admission have all failed, is web_0's large-size gap simply out of reach for this policy family?
- Would small-cache hm_0 respond to giving strongly reused keys more retention, for example protecting keys at the freq cap, or to a small/main split that depends on cache size?
- Can the miss path drop a hash probe, for example by tracking ghost liveness without a separate `ghost_index` lookup or by storing fingerprints?

## Continuation

Keep cycle-5 v002 as the parent. Drop all ghost-admission variants, faster freq aging, and ghost container reshaping. For hm_0 at 16384, test levers that extend main tenure for highly reused keys, or a split that depends on cache size. Treat web_0 at 131072 as low priority unless a genuinely new mechanism appears. For timing, target fewer hash probes per miss. Keep eviction work bounded, and give no timing credit to policy-only diffs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
