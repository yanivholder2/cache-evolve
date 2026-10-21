# Variant v001 Insight

## Helped

- hit-rate: Raising `protected_cap` to 90% of capacity lets more re-referenced keys stay in the protected segment. The scan-heavy trace has a large hot working set, so fewer hot keys are demoted and pushed back to probation. The improvement over the parent is small.

## Hurt

- None observed. The fitness vector reports only hit-rate, so memory and latency effects are unmeasured. The change only alters a constant in the constructor, so it adds no bookkeeping. I could not read the overlay source in this session, so this rests on the hypothesis text.

## Takeaway

On this trace and cache size, giving the protected segment more room beats the parent's smaller share. This matches cycle 0002, where shrinking the share lowered hit-rate. The gain is modest, which suggests the benefit comes from shrinking the probation slice, and that has a limit. Probation must still absorb scan traffic and hold new keys long enough for a second hit to promote them. Pushing past 90% would likely start to lose that scan resistance. The result comes from one trace and one size, so it does not show that 90% is the optimum.
