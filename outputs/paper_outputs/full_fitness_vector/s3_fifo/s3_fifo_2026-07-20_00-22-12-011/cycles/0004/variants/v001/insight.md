# Variant v001 Insight

## Helped

(none)

## Hurt

- hit-rate: Capping the second-chance loop at 16 iterations forces eviction of entries whose frequency is still above zero but whose position in the main deque exceeds the scan budget, discarding hot keys that the unbounded parent scan would have retained.
- per-op-time: Forcibly evicting hot entries beyond the 16-iteration cap drives them back through the miss-and-put path sooner than the parent; the resulting increase in put frequency, combined with each forced-eviction call consuming up to 16 pop_back/push_front pairs, raised put p50, put p99, and get p99 above the parent's values.

## Takeaway

On this Zipfian workload the unbounded second-chance scan in the parent already terminates quickly in practice, because cold entries (freq=0) naturally accumulate at the tail of the main deque and are found within the first few iterations. The SCAN_CAP=16 bound therefore provides no churn reduction in the common case while introducing a new failure mode — forced eviction of still-hot entries once the budget is exhausted — that degrades both hit-rate and latency. The hypothesis that deque churn was the latency bottleneck appears incorrect here; the dominant cost is likely the linear `position()` scan in `get()` and `put()`, which the cap does not address.
