# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three structurally different main-queue and ghost-path tweaks were tried: a one-time reinsertion grace for ghost-admitted entries (v001), protected-end insertion of high-freq small-queue victims into main (v002), and a per-key freq flag in the ghost, with re-admission at freq=2 (v003). None improved hit-rate. v002 is the Winner only because it tied its parent, while v001 and v003 were marginally worse.

## Patterns

- Nothing moved hit-rate. This is the second cycle running in which the best variant merely tied the parent. The S3-FIFO structure looks close to a local optimum on this trace.
- Both variants that gave ghost-admitted keys extra credit (a grace pass, a boosted freq) lost slightly. Ghost-hit information is already used well by plain admission, and extra protection keeps entries that stop being reused.
- The main insertion position (v002) looks like a weak lever. Either few victims reach the threshold, or main's reinsertion logic washes out the position difference.
- Variant authors still could not read the overlay source, so we cannot tell whether v002's change was a no-op. The tie is ambiguous.
- Metadata-size and per-op-time were again missing, and v003 added ghost state with no signal. Evidence is still one trace at one cache size.

## Open questions

- Did v002 alter behaviour at all, or is it equivalent to its parent?
- Is any remaining lever on this trace worth the effort, or are we measuring noise?
- Do these null results hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored so cost can be compared?

## Continuation

Treat v002 as a neutral base; prefer the simplest parent if it is cleaner. Drop ghost-hit bonuses, per-key ghost metadata, and main insertion-position tweaks. The next cycle should validate on a second trace and cache size, and restore the metadata and per-op-time measurements. Only try new levers (such as main-eviction scanning or reinsertion rules) if the diff is checked to change behaviour.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
