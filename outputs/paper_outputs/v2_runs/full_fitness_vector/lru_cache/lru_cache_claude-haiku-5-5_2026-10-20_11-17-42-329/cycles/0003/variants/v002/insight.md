# Variant v002 Insight

## Helped

- metadata-size: Replacing the `Option<u32>` links with a plain `u32` and a `NIL = u32::MAX` sentinel removes the per-link discriminant, which is the entire reduction in `Node<u64,u64>` and the smaller `LruCache` header relative to v003.

## Hurt

- per-op-time: Both `get` and `put` got slower even though the sentinel comparisons replaced `Option` matches on every link access. The hypothesis expected the `Option` match to be a significant share of the hot-path cost; the regression suggests the cost is dominated by the `as usize` arena indexing and the surrounding bounds-checked accesses, which this change leaves intact.

## Takeaway

The sentinel is a clean, policy-neutral way to get the node-size win, and hit-rate stayed identical as expected for a pure representation change. It did not recover the speed lost in v003, so the `Option` discriminant was not the hot-path cost. Any follow-up should look at the arena access pattern itself (for example, fewer repeated `nodes[idx as usize]` lookups in `unlink`/`push_back`) rather than further link-encoding changes.
