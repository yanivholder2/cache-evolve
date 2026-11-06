# Variant v002 Insight

## Helped

None. All four hit-rate cells (web_0 and hm_0, at 16384 and 131072) match the parent to the last digit, including the target cell, web_0 @ 131072.

## Hurt

None on hit-rate. hm_0 did not lose anything either, so the admission gate never rejected real reuse. It never rejected anything often enough to show.

## Takeaway

The overlay contains the change as specified: a sweep capped at one rotation, rejection of the probation item, and a `first_zero_from` prediction. Yet the output is bit-identical to v003. The two policies differ only when main has no counter-0 slot at all at promotion time. In every other case, v003's unbounded sweep stops within its first rotation at the first zero from the hand, which is the same victim the capped sweep picks. So the "hot main" case essentially never happens on these traces, at either size.

The most likely reason is v003's reset-to-0 on promotion. Every promotion places a counter-0 block in main, and on web_0 most promoted blocks are never hit again. That keeps main supplied with cold victims, so the multi-rotation grind described in the rationale was never happening. This refutes H02's premise: promotion pressure on hot main blocks is not what limits web_0 @ 131072. It also means v003's concern about worst-case sweep cost (up to 7 rotations) was mostly theoretical in practice.

Admission gates built on "main has no cold slot" will not help here. Future web_0 work should target which probation-hit blocks get promoted, or which counter-0 main block is chosen, not whether promotion is allowed when main is full of hot blocks. (Caveat: identical results could also mean the evaluation reused the parent's build. Confirming the eval ran this overlay is worthwhile, but the code analysis above already predicts identical results.)
