# Variant v001 Insight

## Helped

- **per-op-time**: Put tail latency (p99) dropped relative to the parent — with the warm-on-access threshold lowered to freq≥2, more items earn CLOCK_BIT after fewer accesses, which changes the distribution of warm flags at eviction time and reduces the worst-case rotation length in `clock_evict_prepare` on the put path.

## Hurt

- **hit-rate**: Lowering the CLOCK_BIT threshold from freq≥3 to freq≥2 over-expanded the warm set on this Zipfian workload; items that have been accessed only twice (a large fraction of the access distribution) now earn clock protection too readily, and `clock_evict_prepare` must rotate through them instead of evicting them, crowding out genuinely cold items and degrading discriminating power.
- **metadata-size**: Item struct grew back to 40 bytes versus the parent's 24 bytes because this variant's overlay only carries `dash_segment.rs` and did not inherit the `item.rs` reduction that was part of the parent's overlay; the threshold change itself is not the cause.

## Takeaway

Lowering the warm-on-access threshold from freq≥3 to freq≥2 confirmed that the midpoint is still too permissive for this workload: the Zipfian distribution places so many items at exactly 2 accesses that the warm flag becomes nearly universal, repeating the saturation problem cycle 8 (freq≥1) exhibited but in milder form. The parent's freq≥3 gate remains the better discriminator; future exploration should look at alternatives other than simple threshold arithmetic — such as a time-decayed counter or a capacity-relative warm-density cap — to protect genuinely hot items without over-inflating the warm set.
