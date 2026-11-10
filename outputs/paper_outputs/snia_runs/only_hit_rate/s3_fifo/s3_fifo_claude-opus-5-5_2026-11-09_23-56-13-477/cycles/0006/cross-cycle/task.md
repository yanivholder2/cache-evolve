# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-variable changes were tried against the baseline: a lower graduation threshold (v001), an O(1) hash-index/slab refactor with the policy left untouched (v002), and a 10/90 small/main split (v003). v001 won, but only barely. Neither policy change moved aggregate hit-rate much: each gained on hm_0 and lost on web_0.

## Patterns

- Both policy knobs (graduation threshold and small-queue size) split the two traces the same way. hm_0 rewards letting keys into main more easily or keeping more main capacity. web_0 penalises it, either through main pollution from keys accessed exactly twice or through medium-reuse keys getting ghosted early. A trace-agnostic constant change looks unlikely to win clearly on both.
- The two policy knobs interact. v003 points to the strict `freq > 1` threshold as what limits a smaller small queue, and v001 points to unfiltered admission as what limits a looser threshold. Each fix seems to need the other's lever.
- What happens to an entry once it is in main doesn't help. Resetting a graduate's freq didn't offset admission pollution, so the cost is paid when a key is admitted to main.
- The refactor kept hit-rate identical to the baseline, so the policy is unchanged. But only hit-rate was measured for this cohort, so the per-op-time and metadata-size effects are still unknown.

## Open questions

- Does v002 actually reduce per-op-time, and does the extra index, slab and stamp state make metadata-size worse?
- Does combining a small queue between 10% and 25% with a threshold of 1 keep hm_0's gain without web_0's loss?
- Can an adaptive signal tell the two traces apart online, such as how much spare room main has or how often keys are reused in the small queue?

## Continuation

Check that per-op-time and metadata-size are being evaluated, and re-measure v002. If it is confirmed faster, use it as the parent for later policy work. Stop trying single-constant tweaks to the threshold or the split. Instead, change the threshold and the small-queue size together, or make graduation into main adaptive (conditioned on main having room or on the observed reuse rate), so that web_0 doesn't regress.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle 0001's v001 (single-reuse graduation). v001 gated the looser threshold on main having room. v002 made the small/main split adaptive, ARC-style. v003 combined the looser threshold with a static 15% small queue. v001 won mostly by undoing its parent's change: main stays full once the cache warms up, so the gate falls back to the baseline two-reuse rule.

## Patterns

- Which keys get into main matters more than queue sizes. Both split variants moved hit-rate by close to noise, and their effect flipped sign across traces and cache sizes. The only clear gain came from stricter admission into a full main.
- Under steady-state pressure, the one-reuse threshold costs hit-rate on web_0 and also on hm_0 at the large size. Its benefit holds only on hm_0 at the small size. Cycle 0001's "hm_0 wants a looser threshold" result was mostly a cold-start or small-cache effect.
- Main occupancy doesn't work as an adaptive signal: it fills and stays full. Any adaptive gate needs a signal that keeps changing in steady state, such as the ghost hit rate or the small queue's reuse rate.
- An unweighted ghost-hit controller probably drifts toward a larger small queue, because the two ghosts differ in size. That brings back the exactly-twice pollution. It also adds a second linear ghost scan on every miss.
- Per-op-time and metadata-size are still unmeasured. Cycle 0001's request to re-measure the hash-index refactor wasn't followed, since all of this cycle's variants descended from the policy line.

## Open questions

- Can a steady-state signal (ghost hit rate, small-queue reuse rate) usefully switch the threshold between 1 and 2, or is the baseline rule simply right for these traces?
- Does the adaptive split move at all? Where `small_size` settles on each trace was never logged.
- How much per-op-time does the extra `main_ghost` scan cost, and how much does the O(1) refactor save?

## Continuation

Treat v001 (essentially the baseline two-reuse rule plus freq reset) as the policy base. Stop tuning the static split, and don't adopt the unweighted adaptive split. Spend one hypothesis on per-op-time and metadata-size, ideally moving the policy onto the O(1) index/slab refactor. If policy work continues, gate admission into main on a signal that keeps changing in steady state, not on main occupancy. A size-weighted ARC step that also logs where `small_size` settles is a secondary option.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle 0002's v001. v001 moved that policy onto the O(1) index/slab structure without changing behaviour. v002 replaced the occupancy gate with a threshold that switches between 1 and 2 based on a windowed ghost-return rate. v003 gave keys returning from ghost one second-chance credit when they re-enter main. v001 won because it was the only variant that didn't regress: its hit-rate matched the parent bit for bit, and both policy variants held or fell in every cell.

## Patterns

- The ghost path doesn't make a good lever in either direction. Using ghost returns as a signal to loosen admission (v002) and rewarding returnees with credit (v003) both lost hit-rate. Under steady-state pressure the ghost mostly holds weak single-reuse keys that v001 refused to graduate, so a ghost return is weak evidence of value.
- v002's switch fired on the wrong traces. It turned lenient on web_0 and on hm_0 at the large size, where strictness had helped, and gained nothing on hm_0 at the small size. Its ratio mixes up how often keys return over a long horizon with what strictness actually costs. The ghost is about 3× larger than the small queue, which inflates the ratio.
- Main stays full, so any extra protection given to one population evicts someone else. v003's credit kept dead keys in main for an extra lap and pushed out entries that would have been hit.
- Across three cycles, the baseline strict two-reuse rule plus the freq reset has held up every time. None of the threshold, split, gate or credit changes has beaten it on all traces.
- Per-op-time and metadata-size have now been missing from the evaluation for three cycles in a row. The cost refactor has been checked for correctness (an oracle test enforces identical hit/miss behaviour), but nobody knows whether it is actually faster.

## Open questions

- How much does v001 reduce per-op-time, and how much does it increase metadata-size, compared with its linear-scan parent?
- Where did v002's threshold settle on each trace? Its threshold-stats output was never captured.
- Would a gate that directly compares the misses avoided by promoting against the main entries displaced by promoting do better than any rate-based proxy?

## Continuation

Use v001 (the O(1) index with the occupancy-gated two-reuse rule) as the parent. Before running more hypotheses, make sure the evaluation measures per-op-time and metadata-size. Drop ghost-rate threshold switching and flat ghost-return credit. If policy work continues, condition it on evidence each key carries with it (for example, its small-queue freq when it went to ghost), or compare the cost of promoting a key against the cost of displacing one directly. Otherwise, accept that the policy has plateaued and spend hypotheses on reducing cost and metadata.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
