# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle 0003's v001 (the O(1) index with the occupancy-gated two-reuse rule). v001 gated the ghost→main fast path on the small-queue freq each key carried into the ghost: freq-0 returnees go back to small with freq 1. v002 was a cost-only change (fast hasher, 8-byte `Loc`, fewer lookups) and left hit-rate bit-identical. v003 gave freq-1 small tails one extra lap in small while main is full. v001 won on a slight summary gain, which came mostly from web_0 at the small size.

## Patterns

- Per-key evidence beats global rate proxies. After two cycles in which ghost-rate switching and flat returnee credit lost hit-rate, gating on each key's own ghost-time freq is the first ghost-path change that improves on its parent. It also confirms that how much a ghost return is worth depends on the trace: weak on web_0, sometimes real reuse on hm_0.
- Both policy variants changed results mainly at the small cache size. At the large size the ghost covers a long reuse window and admitting to main costs little, so moving filtering between ghost, small and main barely changes which key is evicted. Every policy gain so far has been small and limited to the small size.
- v001 and v003 both say the same thing: filtering in small is cheaper than filtering at main admission. They move different populations (freq-0 ghost returnees and freq-1 small tails), so they look complementary rather than redundant. v003's slight dip on web_0 at the large size suggests that when a key's two reuses are spread over a long window, that is weaker evidence than two reuses close together.
- Per-op-time and metadata-size are still missing from the evaluation for the fourth cycle running. The H02 rationale assumed these dimensions were now reported, but v002's fitness vector held only hit-rate. Two cost variants (cycle 0003's v001 and this cycle's v002) are now waiting on measurement.

## Open questions

- Does combining v001's freq-gated ghost path with v003's single relap add up, or do the two compete for the same small slots?
- Can freq-0 returnees win back hm_0 without giving up the web_0 gain? For example, let them into main only when main's tail has freq 0, or give them a stronger initial credit in small.
- How much faster is v002 than its parent, and how much less metadata does it use? Its policy is identical, so any difference comes only from the hasher and the layout.

## Continuation

Use v001 (the freq-gated ghost path) as the policy parent, and port v002's cost changes onto it, since they don't change the policy. Before running more cost hypotheses, confirm that per-op-time and metadata-size are actually in the fitness vector. If policy work continues, try layering v003's relap onto v001, adding a recency requirement for the relapped key's second reuse, or routing freq-0 returnees based on main's tail. Expect small gains limited to the small cache size, and drop anything that relies on global rate signals.
