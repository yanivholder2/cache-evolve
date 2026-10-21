# Variant v002 Insight

## Helped

(No measured dimension improved. The fitness vector again has only `hit-rate`. H02 was a cost-only change aimed at metadata-size, and possibly per-op-time on the miss path, but neither dimension was evaluated. The intended win is real by construction but unmeasured: `Item<u64,u64>` drops its 16-byte `Instant` and shrinks its 8-byte `usize` counter to two `u32` fields, `lfu_counter` and `stamp`. The new in-crate `item_is_compact` test asserts `size_of::<Item<u64,u64>>() == 24`.)

- hit-rate (held, not improved): no movement, which is what H02 predicted. The GCLOCK bucket path never read `timestamp`, and its counter stays within 0..=3, so retyping `CLOCK_MAX`, `admission_counter` and the `lfu_counter` comparisons from `usize` to `u32` leaves the increment, ceiling, sweep, hand placement and ghost-ring logic untouched. The victim sequence on workload-a at 2048 is therefore identical to v001's.

## Hurt

(No measured dimension regressed.)

## Takeaway

On the GCLOCK/LIP line, the per-item wall-clock `Instant` and the wide `usize` counter were dead weight. Removing them is safe for hit rate. The unchanged result confirms that no ClassicLRU/GCLOCK path depends on the timestamp or on the counter's width, so v002 dominates v001 as a base: same behaviour, smaller `Item`, and a relaxed atomic increment instead of `Instant::now()` on every miss. Two things are introduced but not exercised by this eval, and only matter off the winning policy: TimestampLRU now orders by a process-wide `u32` logical stamp, which wraps after about 4 billion stamps, and every `Item::new` now hits one shared atomic, which could become a contended cache line under multithreaded load. Neither the metadata-size claim nor any per-op-time effect can be confirmed until those dimensions are added to the fitness vector.
