# Variant v001 Insight

## Helped

- metadata-size: The rotating start cursor was added without new per-bucket storage, as the hypothesis required, and the item and cache layouts did not grow.
- per-op-time: The put median moved down slightly, which fits a modular start offset adding only a small amount of work to the victim scan; the margin is small enough that it may be noise.

## Hurt

- hit-rate: Rotating the start of the equal-minimum scan lowered hit-rate on workload-a relative to the parent. The hypothesis assumed index-order tie-breaking was concentrating churn harmfully, but the drop suggests that spreading evictions across equal-counter slots does not protect re-referenced keys, because those tied slots hold low-counter entries that were never re-referenced.
- per-op-time: The put tail latency rose, and get tail latency rose even though the get path was unchanged by the hypothesis. The put tail is consistent with the wrapped scan order adding unpredictable branching in the victim search; the get tail movement is not explained by the change and is probably measurement variance.

## Takeaway

The tie-break among slots that share the minimum counter is not a meaningful hit-rate lever on workload-a: the exact-minimum rule is what matters, and the index-order bias it leaves behind is not costing hits. Further work on victim selection should change which slots are eligible or how counters age, not the order in which equal-counter slots are scanned.
