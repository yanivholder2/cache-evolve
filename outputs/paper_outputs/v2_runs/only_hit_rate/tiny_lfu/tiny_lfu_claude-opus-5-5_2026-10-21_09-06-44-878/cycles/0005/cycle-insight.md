# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle-0004 v001 (wide sketch with dedup). Only one of them changed policy. v001 split the residents into an SLRU probation/protected layout and sampled victims only from probation. It won, but by a marginal hit-rate gain. v002 (a narrower sketch with u16 counters and conservative update) and v003 (a semantics-preserving per-op speedup) were built to move metadata-size and per-op-time. Both matched the parent's hit-rate exactly. Because only hit-rate is scored, the evaluation could not see what they were built to improve.

## Patterns

- Structural levers on clean estimates are also close to exhausted. The 8-way min-estimate victim sample was already acting as a soft protected segment, so an explicit SLRU split mostly duplicates it. The split also adds per-hit list work, which is unpriced.
- The sketch is over-provisioned. Half the width with half-size counters plus conservative update left every admission and victim decision unchanged. This confirms that width and conservative update substitute for each other, and that lifetime counts stay far below u16 saturation.
- Hit-rate on workload-a now looks stuck on a plateau around the parent. Three different kinds of change landed on or barely above it. The remaining differences between candidates are in cost axes that are not measured.
- The evaluator has become the bottleneck. This is the fifth cycle with per-op-time and metadata-size unscored. Two of the three variants this cycle are wins that cannot be verified, and the Winners selection cannot reward them.
- New coupling: v003's shortcuts (early stop at estimate ≤ 1, early rejection of newcomers at ≤ 1) are correct only while there is no aging. Any future change that adds decay or reset must re-derive them.

## Open questions

- After per-op-time and metadata-size are scored, which lineage dominates? Possible answers include SLRU, the narrow CU sketch, the hash-cached fast path, or a combination of all three, which should be compatible.
- How far can the sketch shrink (narrower width, or smaller counters) before hit-rate moves?
- Is there any hit-rate left on workload-a, for example from a separate admission window in front of SLRU or a different probation ratio, or has the trace reached its ceiling for TinyLFU-style policies?

## Continuation

Treat hit-rate on workload-a as plateaued and stop spending variants on small policy tweaks. The top priority is getting per-op-time and metadata-size into the fitness vector. Then combine v002's narrow CU sketch and v003's hash-cached fast path with v001's SLRU, and price SLRU's per-hit cost. Probe a narrower sketch only once metadata-size is scored. Any aging proposal must remove v003's ≤ 1 shortcuts.
