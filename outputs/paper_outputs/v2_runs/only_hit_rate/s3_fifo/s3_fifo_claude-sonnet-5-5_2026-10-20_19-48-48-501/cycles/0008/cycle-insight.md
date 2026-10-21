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
