# Variant v001 Insight

## Helped

- **hit-rate**: Expanding `get_from_position` from a single unconditional swap to a loop of up to three unconditional swaps lets items sitting 2–3 slots from the head converge to the protected zone in fewer accesses under Zipfian patterns, reducing the window in which a moderately popular item can be displaced before reaching position 0.

## Hurt

## Takeaway

Tripling the per-access promotion budget (1 swap → up to 3) yielded a measurable hit-rate gain over the parent's single-swap policy without registering any latency cost, since the early-break on `pos == 0` keeps the hot-path at one iteration and the benchmark's 100 ns floor masks any additional swap work. This confirms that the convergence lag for mid-bucket items under a single-step policy is a real source of eviction mistakes on this workload, and that the diminishing returns from extra swaps do not yet appear within a budget of three.
