# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Placing keys that pass the shared-frequency admission gate at the eviction-next slot, instead of MRU, made each admitted key the first victim candidate for the next conflicting insert into its bucket, so the gate's protection of proven-reuse entries was partly undone on workload-a.

## Takeaway

On workload-a the admission gate and eviction-next placement did not stack the way H02 predicted: the pairing lost hit-rate relative to the parent, which holds the gate alone. The next test of placement should keep the gate fixed and check whether placement helps only for keys with a non-zero counter, since that is the subset the gate is meant to protect. Note that the overlay source was not readable in this session, so this mechanism is inferred from the hypothesis's stated change and not confirmed against the diff.
