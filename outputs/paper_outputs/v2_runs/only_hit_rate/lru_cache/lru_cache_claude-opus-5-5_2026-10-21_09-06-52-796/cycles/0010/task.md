# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants came from cycle 8's v003 (`small` at an eightieth, ghost at thirteen-eightieths). Each ran one of the checks cycle 8 asked for:
- **v001** ported cycle 8's `small`-to-`main` credit carryover together with `MAX_FREQ = 7`. It won, by a sliver that could be noise.
- **v002** added a short ghost fed by `main` evictions. Hit rate was nominally flat.
- **v003** ported the long-pending fingerprint ring for the ghost. Hit rate was unchanged, as intended. Its real target, `metadata-size`, was not scored.

## Patterns

- **Each experiment closed its line of work rather than opening one.** Carryover mostly overlaps with the smaller `small`, as predicted. The `main`-fed ghost found almost nothing to recover. So `main`'s eviction of zero-credit keys already rejects keys nearly correctly, and "feed the ghost differently" is answered.
- **On workload-a at 2048, every admission and retention lever we know of is close to used up.** These include: the `small`/ghost split, credit caps, carryover, scan order, entry credits, and now ghost sources. The remaining gains are within noise on one trace and one size.
- **The fingerprint ring is policy-neutral.** It reproduces the parent's admission decisions and removes per-slot key clones. It is a safe base for all later variants. Whether it wins on `metadata-size` depends on the evaluation, not on more code.
- **The variant insights keep proposing an idea that has already failed.** Both v001 and v002 again point at giving ghost-admitted keys a starting credit. That idea lost in cycle 7 and was dropped in cycle 8. Treat it as rejected.
- **The bottleneck is now the evaluation, not the policy.** The fitness vector still has only `hit-rate`. A second cache size or second trace has been asked for since cycle 4 and has still not appeared.

## Open questions

- Does the fingerprint ring actually lower `metadata-size`? Does any S3-FIFO variant come close to the baseline on it, or on `per-op-time`?
- Do the carryover and split gains survive at a second cache size? Do they survive on a second trace?
- In v001, which part caused the sliver: the cap or the carryover? This only matters if it holds up beyond workload-a at 2048.

## Continuation

Use v001 as the parent and port v003's fingerprint ring onto it, as one cheap combined winner. Stop proposing hit-rate tweaks on workload-a at 2048; that search is saturated.

Drop these lines of work:
- the `main`-fed ghost
- credit work on the `small` path
- ghost-arrival starting credits, which lost when tested in cycle 7 and should not be retried

The next cycle should focus on getting `metadata-size` and `per-op-time` scored, and on running at a second size or trace. If the harness cannot do that, flag it rather than inventing new knobs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
