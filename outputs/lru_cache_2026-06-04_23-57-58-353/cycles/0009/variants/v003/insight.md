# Variant v003 Insight

## Helped

- hit-rate: Expanding the protected segment from 80% to 85% gave the hot working set slightly more room, reducing the rate at which warm entries get demoted from protected back to probationary and must be re-promoted.

## Hurt

## Takeaway

Shrinking probationary from 20% to 15% produced a small but real hit-rate gain, confirming that the directional evidence from prior cycles (every protected-reduction hurt) was pointing the right way. The gain is modest, however, which indicates the active working set on workload-a is only slightly larger than what 80% protected can hold — it was not severely capacity-constrained at the old split, just mildly so. Further expansion of protected space may yield similar incremental returns rather than a step-change.
