# Variant v001 Insight

## Helped

- metadata-size: Removing the separate `hit: bool` field and keeping the second-hit flag in a spare bit of an existing `Item` field brings the struct back to its baseline layout, so the padding that `hit` forced is gone.
- per-op-time: Median and tail latency for both get and put improved over the parent, most likely because the smaller `Item` packs more entries per cache line and the bit helpers (`item_hit` / `set_item_hit`) compile to cheap inline mask operations.

## Hurt

- hit-rate: Nothing moved. Packing the flag changes storage only, not the two-hit promotion logic, so the admission-filter benefit carries over unchanged.

## Takeaway

The second-hit admission filter does not need its own field. Its hit-rate benefit survives being folded into spare bits, and doing so removes the metadata cost and some of the latency cost that the explicit `bool` added. For this struct, bit-packing a one-bit flag gives the same behaviour for less space and time.
