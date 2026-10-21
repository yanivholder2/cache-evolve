# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three probes of the sizing-plus-ghost configuration were tried: a stricter small-to-main promotion test (freq>=2, v001), ghosting only main evictees that showed reuse (v002), and an ablation of main-evictee ghosting from the small-queue/short-ghost sizing (v003). v001 is the Winner only because it tied its parent. v002 lost a very small amount, and v003 gained over baseline.

## Patterns

- Sizing alone (small queue ~5%, shorter ghost) delivers the hit-rate gain over baseline. Main-evictee ghosting is not needed for it, so the cycle 0006 gain was mostly sizing.
- Filtering main evictees out of the ghost lost slightly. Evictees that look unreused still carry some re-admission value, and no ghost pollution showed up.
- A stricter promotion gate (freq>=2) again made no measurable difference. This is the second cycle with a tie on promotion strictness, so the gate is either not binding on this trace or the change is a no-op.
- Ghost content and ghost admission selection, like capacity ratios, look saturated. Every cycle's best variant has been a tie or a marginal gain.
- Variant authors still could not read overlay source. Cost dimensions (metadata-size, per-op-time) are still unmeasured. Evidence is one trace at one cache size.

## Open questions

- Does the freq counter ever reach 2 while an item sits in small? If not, the v001 tie is a no-op rather than evidence.
- How much does main-evictee ghosting add on top of sizing? v003 shows it is not required, but not whether it helps or hurts.
- Do the sizing gains hold on a second trace or cache size?

## Continuation

Treat the simplest sizing-only configuration (v003) as the base; it is as good as the more complex 0006 winner, so prefer it unless ghosting main evictees is shown to add something. Drop promotion-threshold and ghost-content filtering variants. Next, validate the sizing on a second trace and cache size, restore metadata-size and per-op-time measurements, and only try new levers if the diff is verified to change behaviour.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three levers were tried: adaptive small-queue sizing from ghost-hit and promotion feedback (v001), a higher frequency saturation cap (v002), and promotion from small to main on the hit path once freq reaches 2, bundled with sizing (v003). v002 is the Winner with a very small gain, likely within noise. v001 lost slightly. v003 improved on its parent, but the gain is confounded with the sizing change.

## Patterns

- Feedback-driven sizing lost to the fixed ~5% small queue. The fixed ratio looks like a real local optimum, and window-based signals were too coarse or noisy to improve on it.
- Raising the freq cap is the untested opposite of earlier cap reductions. It gives at most a marginal gain, so the cap is close to saturated as a lever.
- Hit-path promotion is the first structurally new lever in several cycles. It changes behaviour for certain, unlike the eviction-time gate tweaks that tied. Its gain cannot be separated from the sizing it was bundled with.
- Adaptive control and counter tuning are not paying off. Sizing and promotion timing are where signal has appeared.
- Cost dimensions (per-op-time, metadata-size) are still unmeasured, and hit-path promotion adds work to `get`. Evidence is still one trace at one cache size, and variant authors still could not read overlay source.

## Open questions

- Is v003's gain from hit-path promotion or from the sizing it was bundled with? Parent choice in this cycle makes the two inseparable.
- Is v002's gain real or noise?
- What does promotion on the hit path cost in per-op time?
- Do any of these results hold on a second trace or cache size?

## Continuation

Pursue hit-path actions, using the sizing-only configuration as the explicit parent so the effect of promotion is isolated. Variants to try include a different promotion threshold and a different insertion position in main. Drop adaptive sizing and further freq-cap tuning. Validate on a second trace and cache size, and restore per-op-time and metadata-size measurements.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried. Two isolated hit-path promotion from small to main (freq>=2 on the sizing parent, v001; a stricter freq>=3 gate with freq reset, v002). The third shrank the small queue to ~3% on baseline with a shortened ghost (v003). The promotion probes tied their parents exactly. v001 is the Winner only because of that tie. v003 gained over baseline.

## Patterns

- With sizing held fixed, hit-path promotion made no measurable difference. The cycle 0008 gain from promotion was therefore most likely the sizing it was bundled with.
- Promotion threshold and carried-over freq tie again. This is now the third cycle where promotion strictness or timing leaves hit-rate unchanged. Either eviction-time promotion already catches the same entries, or the gate is rarely binding on this trace.
- A smaller small queue (~3%) still beats baseline when paired with the shortened ghost. Sizing remains the only lever with consistent signal. No 5% comparison exists, so the optimum is not located.
- Exact ties suggest that diffs may not change behaviour. Variant authors still could not read overlay source, so this is unconfirmed.
- Per-op-time and metadata-size are still unmeasured. Evidence is one trace at one cache size.

## Open questions

- Is 3% better than 5%, or just another point on a flat curve? Both were never compared head to head.
- Is hit-path promotion a true no-op on this trace, or a no-op in the code?
- Does any sizing gain hold on a second trace or cache size?

## Continuation

Drop hit-path promotion, promotion thresholds and freq-reset variants; they have tied repeatedly. Next, sweep the small-queue ratio (for example 3%, 5%, 8%) with the same ghost and a shared parent, so the optimum is identified directly. Validate the best sizing on a second trace and cache size, and restore per-op-time and metadata-size measurements.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
