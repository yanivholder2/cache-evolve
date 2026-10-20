# Variant v001 Insight

## Helped

- hit-rate: Tightening the small-queue divisor from `capacity / 16` to `capacity / 24` (~4/96 split) shifted ~43 more slots from the admission filter into the main resident queue, giving hot keys extra second-chance survival and nudging the measured hit-rate above the ~6/94 parent.

## Hurt

## Takeaway

Even at a ~4/96 split the partition-ratio sweep has not reversed: shrinking the admission filter one more step still pays, but the gain over the parent is tiny — a fraction of the parent's own step. The probe confirms the workload tolerates a very thin one-hit-wonder filter without leaking it, yet the diminishing return signals the knee is essentially here; further shrinking risks losing the filtering that keeps the main queue clean.
