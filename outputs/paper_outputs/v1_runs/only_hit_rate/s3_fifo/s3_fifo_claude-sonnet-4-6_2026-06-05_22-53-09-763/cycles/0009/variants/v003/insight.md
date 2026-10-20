# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Lowering the eviction threshold from `freq > 1` to `freq > 0` slightly hurt because the items that reach the main-queue tail at freq==1 on a Zipfian workload are genuinely cold — they were promoted to main at freq==2 but never re-accessed — so granting them an extra recycle pass (1→0, pushed to front) delays admitting fresher candidates without improving their own hit probability.

## Takeaway

The `freq > 1` eviction threshold was already well-matched to the eager-promotion entry frequency: hot items on a Zipfian trace accumulate frequency through in-place `get()` increments and rarely reach the tail with freq==1, while the items that do are cold enough that one additional pass provides no benefit. Extending the recycle window was intended to protect eagerly-promoted items, but those items protect themselves via re-hits; the change instead prolongs the residency of cold squatters, yielding a negligible hit-rate loss and ruling out a further-extended recycle window as a useful direction.
