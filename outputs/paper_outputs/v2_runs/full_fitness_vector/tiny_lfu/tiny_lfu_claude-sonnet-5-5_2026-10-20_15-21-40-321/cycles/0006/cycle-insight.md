# Cycle 0006 Insight

## Cohort outcome

Three sketch-only changes were tried on existing incumbents: a single-hash shift-derived index (v001, on the cycle-0004 v003 parent), a saturation early-exit alone (v002, on the cycle-0004 v001 parent), and conservative-update increments (v003, on the cycle-0005 v002 parent). All three were kept as Winners, but none gave a real improvement. Hit-rate and metadata size stayed flat, and per-op time did not improve.

## Patterns

- The cycle-0005 single-hash speedup did not carry over to the stronger lineage. On this parent it made `get` slightly slower and may have cost a hair of hit-rate, since slicing indices from one hash correlates the rows. The earlier gain was therefore specific to the baseline lineage.
- The early-exit alone did nothing measurable. Counters rarely sit at the cap, so the early-exit rarely fires. Any speedup in the cycle-0005 variant that combined both changes probably came from the hash derivation, and that derivation has now failed to stack.
- Conservative update left hit-rate unchanged. Collision inflation is not what limits hit-rate on workload-a, which is the third sketch-accuracy result (doorkeeper, width, conservative update) to say the same thing.
- Per-op shifts across the cohort were small and in both directions. Most are best read as noise, as in earlier cycles.
- Sketch cost no longer looks like the main per-op term on the current incumbents. Sketch tweaks of every kind (accuracy, width, increment cost) are now exhausted.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Where does `get` actually spend its time on the incumbent: map lookup, list relinking, or sketch estimate? No profile exists.
- Is any remaining gain in the non-sketch hot path (relinking, allocation, bounds checks), or are the incumbents at the floor for this design?
- Is there any hit-rate lever left on workload-a, given that sketch accuracy, victim choice, tie rules, aging, windows and doorkeeping have all failed?

## Continuation

Drop all sketch-internal changes (hash derivation, early-exit, conservative update, width, doorkeeper). Aim the next cycle at the non-sketch hot path of the cycle-0004 v003 incumbent (map access, list relinking, bounds checks), or at a structurally different admission mechanism such as a window or segmented layout, and have each hypothesis name which cost it expects to move. Repeat latency runs before crediting any small delta.
