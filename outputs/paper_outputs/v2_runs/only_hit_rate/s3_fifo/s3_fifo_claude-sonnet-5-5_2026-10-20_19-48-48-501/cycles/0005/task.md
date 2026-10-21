# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three single-knob tweaks were stacked on cycle 0001's v002 (main-victim ghosting): a smaller small queue (v001), a longer ghost queue (v002), and faster freq decay in main eviction (v003). Only v001, the small-queue shrink, beat the parent. The other two were slightly worse.

## Patterns

- The one winner changed capacity allocation (more room for main). It added no new admission or eviction logic. Both losers changed a filter or aging rule, and each made the cache slightly worse.
- The longer ghost loosened admission, which repeats cycle 0001's lesson. Anything that lets weakly-reused keys into main hurts, so ghost length acts as a filter.
- Faster main aging (freq halving) hurt a little. Main residents with high freq are usually reused, so the existing linear decrement looks about right.
- The gain was modest, and it was measured on one trace (LIRS) at one cache size. Metadata-size and per-op-time were again unmeasured, so cost claims remain unverified.
- The structural ratio knob (small/main split) moved the result more cleanly than the rule-level tweaks. Rule-level tweaks have now lost in both cycles.

## Open questions

- Is capacity/10 near the optimum, or does the gain keep growing as the small queue shrinks further? Where does it turn over?
- Does the smaller probation window hurt scan-heavy or other traces, where a larger small queue filters better?
- Would a shorter ghost (the opposite of v002), or a stricter ghost-hit rule, help?
- Why are metadata and timing still missing from the fitness vectors?

## Continuation

Build on v001 (the smaller small queue), and sweep the ratio around it, for example capacity/20 and capacity/8, to find the optimum. Drop the longer ghost and faster freq decay, and don't loosen admission or age main faster. Next, try the opposite direction on ghost size or a stricter ghost-hit rule, and check robustness on a second trace. Restore the metadata and per-op-time measurements.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three single-knob tweaks were tried on the cycle 0002 line: a further smaller small queue (v001), a shorter ghost queue (v002), and a lower freq cap in main eviction (v003). Only v001 won, with a small gain. v002 reported a small gain over its parent but is not among the Winners. v003 was slightly worse.

## Patterns

- The small-queue ratio gain is flattening. This halving gave much less than earlier steps, so the lever is close to exhausted on this trace.
- A shorter ghost helped, or at least did not hurt. That is the opposite of cycle 0002's longer ghost, which hurt. Ghost length now looks like a real admission filter, and shorter means stricter.
- Another attempt to age main entries faster (a lower freq cap) lost a little. Across cycles, ageing and decay changes have lost every time. This workload rewards keeping reuse history.
- Evidence still comes from one trace at one cache size, and every gain is small and may be within noise. Metadata-size and per-op-time were unmeasured for the fourth cycle running.

## Open questions

- Is the v002 shorter ghost additive with the v001 small queue, or does it just overlap with the stricter-admission effect?
- Where does the ghost-length gain turn over? Does it hurt when reuse distances are long?
- Does the very small probation window weaken filtering on scan-heavy or other traces?
- Are the small gains real, or noise from a single trace?

## Continuation

Keep v001's smaller small queue as the base and stop tuning the ratio. Pursue stricter admission instead: a shorter ghost (combined with v001), or a stricter ghost-hit or promotion rule. Drop all ageing and decay variants, including freq cap and halving. Validate on a second trace and cache size, and restore the metadata and per-op-time measurements.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three stricter-admission tweaks were tried: a shorter ghost combined with the small-queue shrink (v001), a promotion gate one freq level stricter (v002), and a ghost-hit fast path into main only if freq > 0 (v003). Only v001 won, by a marginal gain. v002 tied its parent and v003 was clearly worse.

## Patterns

- Admission-side sizing (small ratio, ghost length) now looks saturated. The shorter ghost and the smaller small queue are not additive, and the combined gain is tiny.
- Making the promotion gate stricter had no measurable effect. Either the gate is not a binding lever on this trace, or the change did not alter behaviour.
- Filtering ghost hits hurt the most. A ghost hit is strong reuse evidence, and with a very short small queue, keys sent back through small are evicted before they can prove themselves again.
- "Stricter admission wins" holds only for ghost length. It fails for the ghost-hit path, and it is untested for promotion.
- Only hit-rate was measured, again. Evidence is still one trace at one cache size, and the variant authors could not inspect the overlay source.

## Open questions

- Did the v002 gate change actually alter which keys get promoted, or was it a no-op?
- Is any remaining admission or capacity lever worth tuning, or is the S3-FIFO structure at its local optimum on this trace?
- Do these small gains hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored?

## Continuation

Stop tuning capacity ratios, ghost length, and promotion thresholds, and keep v001 as the base. Keep the ghost-hit fast path permissive, and consider loosening it only for ghost hits, not for general promotion. Explore structurally different levers, such as main-eviction policy or reinsertion rules, and verify that each diff changes behaviour. Validate on a second trace and cache size, and restore the metadata and per-op-time measurements.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
