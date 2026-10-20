# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Changing the shared-frequency admission predicate from strict to non-strict (`incoming >= victim`) admits keys whose counter only ties the probationary victim's, and on workload-a those extra displacements came out slightly net-negative against the parent rather than the gain H03 predicted.

## Takeaway

Admitting on a counter tie does not help on workload-a with unbounded counters: the parent's strict gate already protects victims whose count matches an incoming key, and relaxing it trades those protected entries for equally-counted newcomers without a measurable benefit. Whether ties specifically drive the loss, rather than the change in victim selection pressure, is unverified, since the admitted-on-tie count from the variant notes could not be checked in this pass.
