# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009 v001, the residency-normalised late-hit tie. v001 loosened the late-hit cutoff from a quarter to a half. v002 added aging on a fixed period with ceiling halving and kept double counting. v003 removed the protected-split climber so the window climber acted on every sample. v002 won, but only relative to the cohort. It gained marginally on web_0 small and lost both large cells, hm_0 large most of all. Every variant regressed at least one large cell, so none clearly beats cycle 0009 v001.

## Patterns

- Each change relaxed a guard on the parent's behaviour: a looser tie gate, decayed resident counts, or one less adaptive knob. Each relaxation cost the large cells. The parent's large-cell recovery depends on keeping both ties and count gaps hard for newcomers to win.
- The late-hit cutoff was not too strict. Loosening it hurt web_0 at both sizes, so the parent's web_0 small gap needs a lever other than the tie rule. Every global, floored, reuse-gated or loosened tie change has now lost web_0. If the tie gate moves at all, it should tighten.
- A fixed aging period flipped the capacity-scaling problem without solving it. The period is far longer than the reuse span at the small size, where it is harmless and slightly helpful. At the large size it falls inside the reuse span, and warm residents lose their frequency lead to newcomers that still receive an undiminished double increment. Aging and double counting work against each other whatever the period or rounding.
- The adaptive protected split carries real weight on hm_0 large. That makes it the first structural knob shown to matter for hm_0. Giving the window climber more turns made it worse, not faster, because its step decays per climb.
- Per-op-time and metadata-size are still unmeasured, for the tenth cycle running.

## Open questions

- Does tightening the late-hit gate, or removing the tie exception entirely at large window sizes, strengthen the large cells further?
- Is aging viable at all while double counting stays? Possible tests: aging only with single counting on a fixed period, or a period at a large multiple of capacity.
- Would retuning the window climber's step decay or sample period, while keeping the protected split adaptive, help the large cells?
- Is there an hm_0 lever on the protected side, such as the protected climber's step floor or bounds, since that boundary has now been shown to do work?

## Continuation

Keep cycle 0009 v001 as the base. No cycle 0010 variant clearly beat it, and v002's large-cell losses rule it out as the new base. Drop looser late-hit cutoffs and any aging that keeps double counting. Keep the protected split adaptive. Climber work should retune step decay or sample period directly, not reassign turns. Worth probing next: a tighter tie gate, and the protected-split climber's step and bounds as a lever for hm_0 large.
