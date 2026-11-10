# Variant v003 Insight

## Helped

(None. hit-rate came out identical to the parent on every trace and size. That was the intended invariant, not a gain. It is direct evidence that `main_make_room`/`main_place` reproduce the parent's FIFO-with-reinsertion eviction order exactly, including the order of side effects: release, then ghost, then `alloc`.)

## Hurt

- per-op-time: `put` p99, the metric this hypothesis targeted, went up instead of down, and `put` p50 rose too. The cheaper sweep step did not show up in the results. In both versions, each sweep step is dominated by the random `slab[slot]` load and the `freq` decrement. The parent's `VecDeque` `pop_back`/`push_front` on a fixed-length deque was already just head-index arithmetic on a hot ring, so removing it saved almost nothing. Meanwhile the new path added a `len < main_cap` fill check and an `Option<usize>` hand-off between `main_make_room` and `main_place` on every main insertion.
- per-op-time: `get` also moved up by a similar small amount, even though `get()` is unchanged. Part of the shift across all ops is probably run-to-run noise, so the `put` regression should be read as "no improvement" rather than a reliable slowdown.
- metadata-size: The `S3Fifo` struct grew. The `VecDeque` became a `Vec` plus two `usize` fields (`main_hand`, `main_cap`), a net 8 bytes larger. `Entry<u64,u64>`, the summary dimension, did not change.

## Takeaway

Swapping the `VecDeque` reinsertion sweep for an equivalent CLOCK ring is behavior-preserving, but it does not shorten the `put` tail. A sweep step's cost comes from touching the slab entry, not from moving the queue index. The rationale's premise that `pop_back`/`push_front` costs "two head/tail updates and a different cache line" per step did not hold in practice. So the tail driver for `put` is the number of sweep steps, a policy property set by `MAX_FREQ` and the reuse filters, or the slab access pattern, and not the queue container. Further container-level work on the main queue is unlikely to pay off.
