# Variant v001 Insight

## Helped

- per-op-time: `find_entry_mut` now scans `main` first and falls back to `small` through `position` + `get_mut`. Hot hits stop after one scan of main and no longer walk the small queue first, so `get` p50 improved.
- per-op-time: `put` on a ghost hit now does one `position` scan plus `remove(idx)`. The old `any` scan followed by a full `retain` pass is gone. The tail latency of `put` improved by much more than its median, which suggests the redundant ghost scans were the main source of the slow outliers.

## Hurt

- No fitness dimension regressed. The test suite lost the `small_eviction_promotes_to_main_after_multiple_hits`, `main_eviction_uses_frequency_second_chance` and `repeated_get_clamps_frequency` tests, which weakens the guard on eviction and promotion behavior. The fitness vector does not capture this.

## Takeaway

Reordering and de-duplicating the linear scans in a VecDeque-based S3-FIFO cut latency without touching eviction logic. The unchanged hit-rate and metadata-size confirm the change was behavior-preserving. `put` is still far slower than `get` because a miss must scan main, small and ghost, so the next gains need an index such as a hash map for membership. Removing tests to make room for the refactor is a hidden cost.
