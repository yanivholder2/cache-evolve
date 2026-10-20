# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The doorkeeper rejects the first put miss for any key whose bit is unset, so keys seen once are never admitted; on workload-a this filter dropped keys that recur soon after, and the hit-rate fell against the v002 parent instead of rising as H01 predicted.
- metadata-size: The doorkeeper bitset is an inline `[u64; W]` field on the cache struct, so the struct grew substantially over the parent, while the per-item layout stayed the same as the hypothesis required.
- per-op-time: The put path gained a bit probe and one word-zero clear step per put miss from the amortised clear cursor, and the put median and tail both rose, so the per-miss bookkeeping costs more than the skipped victim scan saves on this trace.
- per-op-time: The get path was meant to be untouched beyond v002, yet get p50 and p99 both rose; the larger struct layout is the most likely cause, since no get-side logic changed.

## Takeaway

Gating admission on a one-shot doorkeeper bit is too aggressive for workload-a: the filter throws away first-touch keys that LIRS-style reuse still needs, so the hit-rate gain H01 expected does not hold and the parent's frequency-only admission is the better baseline. The amortised clear cursor and inline bitset also show that the cost of the gate lands on both the put path and the struct footprint, so any future doorkeeper should be tested with a looser admission rule (e.g. admit on second sighting only after a reuse threshold) before the bookkeeping is retained.
