# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Lowering the promotion threshold to `freq > 0` admitted freq=1 items into the already-small main queue, marginally crowding out items that main's second-chance eviction loop would have retained — and the ghost queue's sharper-but-narrower cold signal (now exclusively freq=0 evictions) did not produce enough incremental ghost-hit promotions on this workload to offset the cost.

## Takeaway

The `freq > 0` threshold change is effectively neutral on this workload: the 10% small queue did prevent a flood of low-quality promotions into main (validating the guard the hypothesis relied on), but the workload lacks sufficient once-accessed-then-reused traffic for the wider admission gate to yield a measurable hit-rate gain. The tighter ghost signal — tracking only cold-evicted keys — did not compensate, suggesting the benefit from ghost precision is already saturated at this ghost-size cap. Widening admission is safe here but not a productive lever; the remaining upside likely lies elsewhere in the eviction policy.
