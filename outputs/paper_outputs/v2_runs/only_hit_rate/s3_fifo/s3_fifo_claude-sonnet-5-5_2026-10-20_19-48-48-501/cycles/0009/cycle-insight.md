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
