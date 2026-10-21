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
