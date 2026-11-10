# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004's v001 (freq-gated ghost path). v001 added the one-time small relap on top of it. v002 split freq-0 ghost returnees by their own ghost age, so quick returns go straight to main. v003 ported the cost-only changes and left hit-rate bit-identical. v002 won. It is the first variant in this lineage to gain on both traces at the small size without losing in any cell.

## Patterns

- Per-key evidence keeps paying off, and how recently a key returns is a stronger signal than whether it was reused in small. A key's ghost age costs nothing to read, since stamps already count ghost insertions, and it separated hm_0's real reuses (squeezed out of a small queue) from web_0's late, one-off rereferences. Global rate proxies never managed that split.
- Both policy variants depend on the "reused once" signal the ghost gate uses. v001's relap resets freq to 0, which erases that signal, so failed relaps lost their direct ghost→main return. That probably cancelled the gain on hm_0 at the small size. Mechanisms layered onto the ghost gate have to preserve the per-key evidence it reads.
- Windows that scale with `small_size` reach the large cache size. Before this cycle, policy gains were limited to the small size. Here v001's relap and v002's age window both moved cells at the large size, because their windows grow with the small queue. web_0 at the large size remains the one cell neither variant moved.
- Per-op-time and metadata-size are still not in the fitness vector, for the fifth cycle running. This holds even though H03 assumed the leaderboard now reports them. Three cost-only variants are now waiting on measurement, and v003 is the one that combines the cost changes with the winning policy line.

## Open questions

- Does widening or narrowing the age window (`GHOST_RECENT_AGE_DIVISOR`) move web_0 at the large size?
- Does v001's relap add to v002's gain if a failed relap is ghosted with freq 1 (or marked as relapped), so it keeps its ghost→main return?
- How much do the cost changes actually save once the cost dimensions are measured? On 8-byte keys, alignment may cancel the ghost-deque saving.

## Continuation

Use v002 (the age-gated ghost path) as the policy parent, and port v003's cost changes onto it, since they leave hit/miss behaviour identical. Next, sweep the age window, and retry the relap on top of v002 with failed relaps keeping their freq-1 ghost signal. Drop freq-resetting relaps. Do not spend further hypotheses on cost work until per-op-time and metadata-size actually appear in the fitness vector.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005's v002 (the age-gated ghost path), and all three changed its ghost or small routing. v001 restacked the one-time relap, this time ghosting failed relaps at freq 1. v002 widened the freq-0 recency window to two small-queue turnovers. v003 added an age ceiling on freq ≥ 1 returnees. v003 won, but only because it moved the least: it rose slightly on hm_0 at the large size and dipped slightly in every other cell. None of the three cleanly beat the parent, so this cycle mapped where the policy's boundaries are rather than finding a new gain.

## Patterns

- The parent's ghost gate is close to a local optimum on all three axes probed. Stacking a second reward for one reuse (v001), loosening the recency floor (v002) and tightening admission for freq-1 returnees (v003) each gave back hit-rate on web_0, which is the trace the parent's gate was tuned against.
- The two signals do different jobs. Ghost age is useful as a floor on freq-0 returnees, and about one small-queue turnover is the right boundary: going to two turnovers lets in web_0's late one-off rereferences. Age adds little as a ceiling on freq-1 returnees, because a reuse in small is still good evidence late in the ghost window.
- Once failed relaps keep their freq-1 ghost signal, the relap duplicates what the ghost path already does, and it costs small-queue capacity. The cycle 0005 relap gain on web_0 at the large size came from the old freq-0 base and did not carry over.
- web_0 at the large size has now resisted every ghost and small routing change, whether it loosened or tightened admission. Whatever limits that cell, it isn't how ghost returnees are routed.
- Every small-queue re-admission (relap, re-routed returnee) pushes another resident out of small. At the small cache size this displacement keeps costing more than whatever main pollution it avoids.
- Per-op-time and metadata-size are still absent from the evaluation, for the sixth cycle running.

## Open questions

- What actually limits web_0 at the large size: main's eviction order, the small/main split at that size, or first-admission filtering of keys never seen before?
- Would a window narrower than one turnover sharpen the web_0 / hm_0 split, or is one turnover already at the boundary?
- Is hm_0's modest large-size gain under v003 real, or is it noise?

## Continuation

Keep cycle 0005's v002 policy, with the age window at 1×, as the policy base. Drop relap stacking, windows wider than one turnover, and age ceilings on freq ≥ 1 returnees; the ghost-path routing axis looks exhausted. If policy work continues, move off the ghost path and target main's eviction or first-admission handling for web_0 at the large size, and try at most one narrower-window probe. Otherwise, port the pending cost-only changes and push for per-op-time and metadata-size to be measured.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
