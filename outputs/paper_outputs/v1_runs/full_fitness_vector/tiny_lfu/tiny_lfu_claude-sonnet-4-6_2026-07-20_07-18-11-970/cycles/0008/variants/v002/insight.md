# Variant v002 Insight

## Helped

- **hit-rate**: Promoting probationary items to the protected chain on first access prevents the admission gate from re-competing proven-hot residents against cold arrivals, yielding a modest but real gain on the stable Zipfian trace.
- **per-op-time**: Get p50 latency dropped because items already in the protected chain are simply re-positioned within the hot-chain rather than being unlinked and re-linked through the full probationary chain; on a Zipfian workload most hits land on protected items, so the cheaper re-positioning dominates the benchmark. (Put p50 rose slightly due to the extra promotion and demotion chain-management overhead on writes.)

## Hurt

- **metadata-size**: The `prot_flags: Vec<bool>` (one slot per capacity entry) plus four new struct fields grew `TinyLfuCache` from 152 to 224 bytes — a 47 % increase in per-cache struct size; this cost is real but invisible to the dimension summary, which keys on `Node` size and was unchanged.

## Takeaway

Adding a 20%-capacity protected segment delivered the expected hit-rate improvement, but the gain is modest because the baseline admission gate already filters most low-frequency evictions — the protected zone adds value only at the margin by shielding items that have earned at least one cache hit from the victim pool. The structural cost is a materially larger `TinyLfuCache` struct (driven by the `prot_flags` Vec) and a small put-latency regression, both of which would compound if capacity or write rate were higher than in this trace.
