# Variant v001 Insight

## Helped

## Hurt

- **hit-rate**: The two-hit SEEN_BIT admission filter keeps first-hit entries at their existing FIFO position in the probationary queue rather than graduating them immediately, so those entries can be evicted before accumulating a second access — removing the protection that the parent's immediate single-hit graduation provided.

## Takeaway

Reducing the first probationary hit to a single `prev |= SEEN_BIT` bit-OR (no relinking) successfully avoids any per-op-time regression, confirming the cost of the operation is negligible. However, the stricter two-hit admission policy trades hit rate for that structural economy: entries that would have graduated to protected on their first access under the parent now stay exposed in the eviction-eligible probationary FIFO, and the workload has enough single-access items that a meaningful fraction are evicted before reaching the second hit needed to graduate.
