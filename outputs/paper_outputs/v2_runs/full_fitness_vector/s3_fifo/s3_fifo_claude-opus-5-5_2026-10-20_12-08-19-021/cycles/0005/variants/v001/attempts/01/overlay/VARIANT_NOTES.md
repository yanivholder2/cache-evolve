# cycle/0005/v001 — H01 notes

## Change (single lever)

`src/s3_fifo/mod.rs`: the ghost-hit shrink guard changed from
`self.small_size > (self.capacity / 16).max(1)` to
`self.small_size > self.small_floor()`, where
`small_floor() = (capacity / SMALL_FLOOR_DIV).max(1)` and `SMALL_FLOOR_DIV = 64`.
At capacity 2048 the floor drops from 128 to 32.

Nothing else in the policy changed from cycle/0004/v003. The upper clamp `capacity / 2`, the
graduate-while-main-full grow trigger, `freq > 1` graduation, `MAX_FREQ`, the fix-up
evictions, and the ghost ring are all the same. `ghost_cap` and `ring_len` still come
from the initial `main_size` (1536 at capacity 2048). The new test
`small_floor_is_capacity_over_64` pins that.

## Instrumentation (test-only, plus a public accessor)

- `pub fn split(&self) -> (usize, usize)` returns `(small_size, main_size)`.
- `#[cfg(test)] SplitStats` counts:
  - `shrinks`: ghost hits that shrank small.
  - `floor_blocks`: ghost hits where the floor guard blocked a shrink.
  - `grows`: graduate-while-main-full events that grew small.
  - `min_small`: the smallest `small_size` seen.
- `#[cfg(test)] floor_div` lets the in-file test replay the parent clamp (16) next to the
  new one (64). Non-test builds use the constant, so the struct layout and metadata size
  are unchanged.
- `h01_split_report` replays a deterministic Zipf stream (400k ops, capacity 2048, three
  `(n, alpha)` settings) under both clamps. It prints hit-rate, settled `small_size`,
  `min_small`, `shrinks`, `floor_blocks` (with its percentage of ghost hits) and `grows`.

Run it with:

    cargo test --features test-utils --lib h01_split_report -- --nocapture

## Measured values

**Not yet collected.** This authoring session had no shell access, so the
overlay was neither compiled nor run. Fill in the settled `small_size` and `floor_blocks`
for `cap/16` and `cap/64` from the command above. If `floor_blocks == 0` and
`min_small > 128` under `cap/16`, the parent never reaches its floor and this lever does
nothing on that stream.
