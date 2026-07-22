# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Doubling `bucket_size` while halving `num_of_buckets` over the same total capacity gave each SLRU bucket more promotion slots but left fewer independent sets, so the coarser hash spread lost more than the deeper per-set associativity gained — hit-rate fell below the parent's verbatim harness split.
- per-op-time: `get`'s p99 tail doubled because `get_position`'s linear scan and the SLRU rotations grow with `bucket_size`; the wider bucket lengthened the worst-case probe exactly as the hypothesis warned, even though the p50 summary held at the floor.

## Takeaway

This is the first direct test of the geometry lever, and it falsifies H01: widening associativity 2× did not break the segmented hit-rate ceiling — it regressed it, showing that at this size the SLRU incumbent needs set *diversity* more than per-set *depth*, so the ceiling is not simply a bucket-width problem. The trade also surfaced the predicted scan cost as a doubled `get` tail without moving the median.
