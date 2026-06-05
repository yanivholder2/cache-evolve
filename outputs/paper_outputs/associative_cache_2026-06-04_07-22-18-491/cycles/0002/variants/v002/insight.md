# Variant v002 Insight

## Helped

*(no dimensions helped)*

## Hurt

*(no dimensions hurt)*

## Takeaway

Capping `lfu_counter` at 16 produced no change in hit-rate on workload-a, indicating the trace's hot set is stable enough that counter saturation never creates a blocking problem at this cache size: the same items dominate the eviction order whether their counters are unbounded or capped, so the scenario the hypothesis targeted — a formerly-hot item's inflated counter preventing a rising item from surviving eviction — does not occur on this workload. The cap is not harmful, but the trace lacks the hot-set churn needed to reveal any benefit from it.
