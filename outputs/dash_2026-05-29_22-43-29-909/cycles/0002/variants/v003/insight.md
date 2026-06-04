# Variant v003 Insight

## Helped

- (none) The change compiled and ran — unlike cycle-1's field-removal attempt — but no fitness dimension improved over the parent.

## Hurt

- (none) hit_rate and per_op_time were left untouched by design (the narrowed fields are never read on the ClassicLRU eval path), and metadata_size held at its parent value rather than regressing.

## Takeaway

The overlay landed the full narrowing — `lfu_counter` to `u32` and `Instant` replaced by a `u32` logical clock, policy ordering preserved — and the crate built and evaluated, yet the `metadata-size` reading for `Item<u64,u64>` stayed exactly at the parent's: the predicted shrink never registered. So `sizeof(Item<u64,u64>)` as measured here is decoupled from the overlaid `src/shared/item.rs` — editing `Item` field widths in this overlay is not the lever that moves the metadata axis. Before spending another attempt on field narrowing, find where that `Item` byte figure is actually sourced, because correctly shrinking the struct in the overlay does not change the number the metric reports.
