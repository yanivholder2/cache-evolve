# Variant v002 Insight

## Helped

- metadata-size: Replacing both `Option<usize>` links (16 bytes each) with `u32` sentinel indices halved `Node<u64,u64>` and shrank the whole `LruCache` struct, landing exactly the node footprint the hypothesis predicted.

## Hurt

- per-op-time: The `u32`/`usize` conversions now sprinkled across every arena access (`idx as u32` on lookup, `prev as usize`/`next as usize` inside `unlink`/`push_back`) added per-operation overhead, roughly doubling get and put latency rather than improving cache-line use as hoped.

## Takeaway

The `u32` sentinel encoding is a clean, behavior-preserving way to win metadata-size (hit-rate stayed byte-for-byte identical, confirming it is pure representation), but the constant index-width casts at every link access made per-op-time worse — the smaller node did not pay for itself in locality. A future variant should keep the narrow links while indexing the arena without per-access casts.
