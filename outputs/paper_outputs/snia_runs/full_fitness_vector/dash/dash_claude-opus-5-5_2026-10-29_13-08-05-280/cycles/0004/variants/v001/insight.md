# Variant v001 Insight

## Helped

- metadata-size: Replacing the 16-byte, align-8 `Instant` with a `u32` tick next to the `u8` `lfu_counter` drops `Item<u64,u64>` to key(8) + value(8) + tick(4) + counter(1) = 21, which rounds up to 24. v002's insight predicted exactly this. The un-ignored `item_u64_u64_smaller_than_40_bytes` test now asserts it. `Dash<u64,u64,DashSegment>` was already 24, so the Item is no longer the larger of the two measured types.

## Hurt

- per-op-time: get got slower at both p50 and p99, and the code change doesn't explain it. The evaluated policy is ClassicLRU, and its `get_from_position` (remove + push) never reads or writes `timestamp`. Shrinking the moved element from 40 to 24 bytes should, if anything, make that shift cheaper. Two things make this look like noise: v002's insight already flagged its own get improvement as probably noise, and this result roughly undoes it. put p50 rose slightly, and that does have a plausible mechanism. Every `Item::new` now runs `fetch_add` on a single process-wide `static TICK: AtomicU32`. Under a multi-threaded benchmark, that one cache line bounces between cores, while the old vDSO `clock_gettime` touched no shared state. The hypothesis expected the atomic to be cheaper. Under concurrency it is probably not.

## Takeaway

The layout itself is the win. Changing `timestamp` to a `u32` is what finally lets the earlier `u8` counter narrowing pay off, and hit-rate stays identical because ClassicLRU never reads the field. A global `AtomicU32` is a poor choice for the clock, though, because it adds a contended write on every insert to buy a field the evaluated policy ignores. A follow-up could keep the 24-byte layout and remove the shared counter. Options are a per-segment or per-bucket tick, or skipping the tick entirely for policies that don't read it. Re-running the per-op benchmark would also show whether the get regression is real.
