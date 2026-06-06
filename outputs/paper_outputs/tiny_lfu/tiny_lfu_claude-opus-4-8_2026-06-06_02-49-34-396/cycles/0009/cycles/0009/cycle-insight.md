# Cycle 0009 Insight

## Cohort outcome

Acting on Cycle 0008's "hunt a new structure or bank cost wins" mandate,
the three variants split across surfaces: a deeper cost cut (sketch depth
4→2, v001), a genuinely new eviction structure (SLRU probation/protected
split, v002), and a microarchitectural increment swap (branchless
saturating add, v003). Every variant missed its primary claim — the two
cost probes regressed per-op-time and v001 also lost hit-rate; the SLRU
moved hit-rate only a sliver while charging footprint and per-hit work.
They are Winners only in the narrowest cost-axis sense.

## Patterns

- The increment loop body is confirmed a dead cost surface. Both edits
  that touched the per-row bump — removing rows (v001) and removing the
  branch (v003) — made the hot path *slower*, not faster. The saved work
  sits below noise, and on this saturating, well-predicted counter
  distribution the branchless rewrite forces an unconditional store where
  the branch let the common path skip it. Per-op gains must come from the
  eviction walk, not the sketch increment.
- Sketch depth is not actually accuracy-saturated at the bottom end.
  Cutting 4→2 leaked aliased estimates into admission and dropped
  hit-rate: the parent's extra rows were still doing real
  collision-correction. Depth=4 is the operating floor, not a saturated
  knob to trim.
- A real new eviction structure still hits the same ceiling. SLRU
  shielded twice-proven entries from one-hit newcomers exactly as
  intended, yet the existing `incoming_freq > victim_freq` admission gate
  already filters those newcomers — so list structure contributed almost
  nothing. The hit-rate ceiling is set by admission, not by list shape.

## Open questions

- With admission confirmed (again) as the binding constraint, is there
  any admission-side edit that isn't a margin/constant tweak — already
  ruled inert — that could move hit-rate?
- Are there *any* cost-axis savings left to bank that don't tax the hot
  path, now that depth, the increment loop, and list structure all carry
  a per-op or hit-rate cost?

## Continuation

Close three more threads: sketch depth has a hard floor at 4 (do not
trim it), the increment loop body is a confirmed non-surface for per-op
speed (neither fewer rows nor branchless bump helps), and SLRU/segmented
eviction is ruled flat because admission already does its job. Cycle 0008
already retired width, conservative update, and the admission margin.
The hit-rate-mover search is now exhausted across estimate, geometry,
admission-margin, and eviction-structure surfaces — treat hit-rate as
saturated at the incumbent base. Next cycle should either probe the one
untouched lever (the eviction *walk* / victim sampling, named by v003 as
the remaining cost surface) for tail/per-op savings, or stop hunting
hit-rate entirely and bank fixed-hit-rate cost wins.
