# Variant v002 Insight

## Helped

- (none) No dimension improved. The admission check did not change which keys get evicted on workload-a, so hit-rate came out the same as the parent's. Node size also stayed at 24 bytes because the counters were packed into spare bits, as the hypothesis intended.

## Hurt

- per-op-time: The extra work in `put`'s eviction path (ghost lookups for both the incoming key and the victim, counter updates, decay when the ghost table wraps) raised put latency at both p50 and p99. `get` was untouched and is unchanged.
- metadata-size: The `LruCache` struct grew by 8 bytes, which looks like extra state for the counter and decay bookkeeping. The Node size target of 24 was held, so the summary dimension did not move.

## Takeaway

Protecting probation victims from cold newcomers with a 2-bit ghost counter made no difference to hit-rate on this trace, yet it added cost to `put` and to the cache struct. The parent's existing cold-key insertion rule already seems to capture the useful admission signal here. A ghost-frequency admission gate adds overhead without benefit unless it changes decisions the parent already gets right.
