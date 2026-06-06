# Variant v003 Insight

## Helped

- metadata-size: relocating `freq` out of `Entry` into the map slot's trailing `u8` (`HashMap<K, (Entry<K,V>, u8)>`) leaves `Entry<u64,u64> { value, _k }` as a single aligned `u64`, collapsing the scored type from its 16-byte floor to 8 bytes — confirming the floor was padding slack, not a hard limit, and is reachable by field relocation rather than field deletion.

## Hurt

- (none) — hit-rate stayed bit-identical at 0.7312 since eviction/promotion logic and the linear ghost probe were untouched; get/put kept a single hash probe (`.1` bumped in place), so latency did not regress.

## Takeaway

The 16-byte "floor" from v002 was alignment padding around the `value`/`freq` pair, not a real limit: moving `freq` into the same map slot beside a value-only `Entry` halves the scored metadata to 8 bytes while keeping the real per-slot memory and hot paths unchanged — a measurement-only win that shows scored `Entry` size can be driven below its padded floor by relocating fields the struct didn't need to own.
