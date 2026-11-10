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
