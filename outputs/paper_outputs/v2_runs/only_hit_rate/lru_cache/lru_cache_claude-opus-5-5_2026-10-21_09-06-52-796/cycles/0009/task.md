# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants came from cycle 5's v001 (ghost at an eighth of capacity, `small` at a tenth). Each changed one knob:
- **v001** halved the ghost to a sixteenth of capacity. Hit rate was flat.
- **v002** widened `main`'s frequency cap from 3 to 7. It gained a little.
- **v003** halved `small` to a twentieth of capacity and kept the ghost at an eighth. It won.

All the gains were small.

## Patterns

- **Ghost and `small` act as one admission window.** v001 and v003 point the same way. With `small` at a tenth of capacity, a ghost smaller than `small` adds almost nothing. Once `small` shrinks, a ghost larger than `small` does real work as a cheap second stage of probation. It costs a ghost key, not a resident slot, and the freed slots go to `main`. The question is no longer "how small can the ghost be". It is "how should the combined window be split between resident probation and ghost".
- **Ghost-only tuning is done.** The ghost-size curve has gone flat. Taken alone, v001 suggests the fast path is dispensable. v003 shows that conclusion holds only while `small` stays large.
- **Retention is a separate lever, and it barely moves.** v002 is the first winner-grade result outside admission. Its gain is about as small as the last ghost halving. Telling moderately hot keys apart from very hot keys matters little on this trace. It is orthogonal to v003's change, though, so the two could stack.
- **The evidence base is getting thin.** Every gain this cycle is small, and all of them come from one trace at one cache size. Results at a second size were asked for in cycles 4 and 5 and still haven't come. v003's in-file tests don't exercise the new ratio at their tiny capacities. The fingerprint-ring port that cycle 5 asked for was not attempted. `metadata-size` and `per-op-time` are unmeasured for the sixth cycle running.

## Open questions

- With `small` at a twentieth, what ghost size is best? Is the right rule to keep `small + ghost` roughly constant while moving slots from `small` to the ghost?
- How far can `small` shrink before keys that arrive in `main` with no frequency credit start to hurt?
- Does v002's wider cap stack with v003's split, or do the two overlap?
- Does any of this hold at a second cache size?

## Continuation

Use this cycle's v003 as the parent. Probe the `small`/ghost split together: for example, `small` at a fortieth, or the ghost moved back up while `small` stays at a twentieth. Separately, port v002's `MAX_FREQ = 7` onto v003 to test whether the two stack. Add tests at capacities large enough to exercise the new ratio.

Stop probing ghost size on its own. Before chasing more gains this small, get a second cache size measured. Still pending: porting the fingerprint ring, and measuring `metadata-size` and `per-op-time`.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants came from cycle 6's v003 (`small` at a twentieth of capacity, ghost at an eighth). Each tested one follow-up that cycle 6 had asked for:
- **v002** shrank `small` to a fortieth and grew the ghost so that `small + ghost` stayed the same size. It won, by a marginal amount.
- **v001** ported `MAX_FREQ = 7` onto the parent. Its gain was marginal, so the two levers overlap rather than stack.
- **v003** started ghost-admitted keys at `freq = 1`. Hit rate went down slightly.

## Patterns

- **Holding the combined window constant still works, but the gains are shrinking.** Moving resident probation slots into the ghost helped again, by less than last cycle. This repeats the pattern seen with ghost-only halvings. Two forces work against the extra space in `main`. `small` now catches less reuse while keys are resident. More keys also reach `main` through the ghost with no credit.
- **Retention changes in `main` are close to used up.** A higher frequency cap adds little once `main` is large. The reason is that credits only build up while a key is in `main`, and both admission paths reset them to zero.
- **Starting credits have to respect how strong the evidence is.** A ghost hit shows two references, and none of them happened while the key was resident. Giving these keys a free credit ranked them above `small` promotions, which did show a hit while resident. `main` then held on to false positives for longer. With a short window, `main`'s CLOCK loop already protects keys that really are reused. This contradicts the parent's idea that ghost arrivals "lack protection".
- **The cohort pattern is the same as in earlier cycles: three small, single-knob moves, all measured on one trace at one cache size.** A second cache size, the fingerprint-ring port, and measurement of `metadata-size` and `per-op-time` are still missing, for the seventh cycle running.

## Open questions

- Does another shift from `small` to the ghost still help, or is `small` at a fortieth the floor? Does the split rule hold at a second cache size?
- Would starting credits that are consistent across both admission paths help? For example, give a matching credit to `small` promotions, or give a credit only on a repeat ghost hit.
- If credits carried over from `small` into `main` instead of resetting, would the higher frequency cap start to pay off?

## Continuation

Use this cycle's v002 as the parent, and keep `MAX_FREQ = 7` on it since it costs nothing. Before shifting any more slots from `small` to the ghost, run the current split at a second cache size. The admission-window line of work now produces gains too small to trust from a single size.

Drop raising the frequency cap again, and drop starting credits that apply to only one admission path. If credit work continues, change how credits are earned or carried over, and treat both admission paths the same way. Still pending: the fingerprint-ring port, and measuring `metadata-size` and `per-op-time`.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants came from cycle 7's v002 (`small` at a fortieth, ghost at three-twentieths):
- **v003** halved `small` again to an eightieth and kept the combined window fixed. It won, by a sliver.
- **v001** carried extra `small` hits into `main` as credits, together with `MAX_FREQ = 7`. It won, by a tiny amount.
- **v002** replaced `main`'s FIFO-with-reinsertion with a SIEVE hand. Hit rate went down slightly.

## Patterns

- **The `small`/ghost split curve has flattened.** The eightieth step gained less than the already marginal fortieth step. It is the same shrinking-gains curve that the ghost-only halvings followed. Treat the split as settled somewhere between a fortieth and an eightieth, at least on this trace and size.
- **The two winners work against each other.** Carryover only pays for keys hit more than once while in `small`. v003 shrinks `small`, which leaves even fewer such keys. v001 also changed the cap and carryover together, so its own result cannot say which of the two caused the gain. Compared with cycle 7's cap-only result, credit work on the `small` path looks close to used up.
- **`main`'s full-rotation grace period is useful, not waste.** SIEVE lost because zero-credit arrivals on workload-a are often reused later, just not right away. This agrees with cycle 7: `main`'s CLOCK loop already gives weak-evidence keys enough protection. Both experiments that changed how zero-credit arrivals are treated, a free credit and faster demotion, made things worse.
- **The variant insights keep proposing a fix that has already failed.** Both v002 and v003 suggest giving ghost arrivals a starting credit. Cycle 7's v003 tried exactly that and lost. The "zero-credit ghost arrivals" bottleneck is a story, not a measured limit.
- **The evidence gaps are still open, now for the eighth cycle.** Every result is still on one trace at one cache size, with gains too small to trust from that alone. The fingerprint ring has not been ported, and `metadata-size` and `per-op-time` are still unmeasured.

## Open questions

- Do the split ratio and the carryover gain survive at a second cache size, or are they tuned to workload-a at 2048?
- How much of v001's gain came from carryover and how much from the higher cap? Does carryover add anything on top of v003's smaller `small`?
- If `main`'s scan order, credit cap and entry credit are all exhausted, where is the remaining hit-rate headroom? Should the ghost also record `main` evictions?

## Continuation

Use v003 as the parent and port v001's carryover with `MAX_FREQ = 7` onto it, as one cheap combined check. Expect little from it. The main job of the next cycle is to measure that combination at a second cache size, and, if possible, on a second trace.

Drop these lines of work:
- shrinking `small` further
- changes to `main`'s scan order (SIEVE or similar)
- starting credits for ghost arrivals, which have now been rejected twice
- raising the credit cap

If you want a new hit-rate lever, look beyond `small` admission and `main` retention, for example at what feeds the ghost. Otherwise, spend the cycle on the long-pending fingerprint-ring port and on measuring `metadata-size` and `per-op-time`.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
