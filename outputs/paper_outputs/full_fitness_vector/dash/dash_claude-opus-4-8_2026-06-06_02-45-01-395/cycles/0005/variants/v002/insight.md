# Variant v002 Insight

## Helped

- hit-rate: Storing buckets as concrete `Vec<DashBucket>` instead of `Vec<Box<dyn Bucket>>` was behaviour-preserving as intended — slot/cascade logic stayed bit-for-bit, so hit-rate held exactly.
- metadata-size: The change touched only storage type and dispatch, not `Item`/`Dash` layout, so the metadata footprint was unaffected.

## Hurt

- per-op-time: Inlining `DashBucket` into the `Vec` (each element now carries the full `items` Vec + `max_size` + boxed policy + bool rather than one pointer) widened the per-element stride and worsened the locality of the index-then-touch access in `get_position`/`put_at`; every op metric regressed — get p50, get p99, and especially put p50 — so removing the outer `Bucket` vtable cost more than it saved.

## Takeaway

Collapsing the outer `Box<dyn Bucket>` indirection is the inverse of the inner-policy win banked in the parent: replacing one pointer per slot with a fat inline bucket struct fattens the `Vec` stride and degrades cache behaviour on the indexed bucket access, so the saved vtable dispatch is swamped by worse locality. Outer-layer monomorphization is not a free structural lever here — the heap pointer was buying compactness, and devirtualizing it regresses all per-op latencies while leaving hit-rate and metadata untouched.
