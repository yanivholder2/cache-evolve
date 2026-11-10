# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
