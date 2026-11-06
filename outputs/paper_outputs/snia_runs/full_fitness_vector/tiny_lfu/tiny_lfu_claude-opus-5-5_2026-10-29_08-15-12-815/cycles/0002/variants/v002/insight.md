# Variant v002 Insight

## Helped

- per-op-time: Every `increment_hashed` / `estimate_hashed` now lands in a single 64-byte-aligned `Block` chosen from `h >> 32`, so each sketch access costs one cache miss instead of v003's four misses spread `width` apart. That is the get-tail cost the parent's insight identified. Get p50 and get p99 both drop sharply, and the earlier pattern where the median improved while the tail got worse is gone.
- per-op-time (put tail): The admission path on `put` does one increment and two estimates (the incoming key and the victim). With the blocked layout, each of these touches one line instead of four, so put p99 falls about as much as get p99. The `#[cold]` halving sweep over packed `u64` words with `& 0x7777…` runs rarely and is cheap enough that it doesn't show up in the tail.
- metadata-size: Packing 4-bit nibbles into `u64` words keeps the counter count of `depth × width` but shrinks the sketch heap by the 32-to-4-bit packing ratio, well under the 64 KiB budget at cap=1024. The headline summary (per-node size) doesn't move, because `Node` and `TinyLfuCache` are untouched.
- hit-rate: This wasn't part of the claim, but the aging reset (halving every counter after `10 × capacity` additions) lets admission forget stale frequency. Both small-cache configurations (16384 entries on web_0 and hm_0) gain clearly: hot-but-old victims no longer pin their slots against newly popular keys. v003 had no aging at all, so this is new behavior, not just a layout change.

## Hurt

- hit-rate (web_0 @ 131072): This is the one configuration that slipped slightly. The likely causes are that 4-bit counters saturate at 15 and so stop ranking very hot keys against each other, and that the fixed `10 × capacity` sample period makes a large cache forget long-range frequency. Another possibility is correlated collisions, since all four of a key's counters share one block, which makes the four rows less independent than v003's. The change is small and may be noise. Which cause dominates wasn't isolated.

## Takeaway

Once hashing was cheap, sketch memory locality was the real limit on per-op latency. Putting a key's four counters in one aligned cache line fixed the get tail that v003 left behind, and the 4-bit packing also cut the sketch heap. Adding periodic halving along with the narrower counters was not neutral for hit-rate: it clearly helps small caches by aging out stale frequencies, but it may cost a little on large caches with long-range reuse. So the sample-period multiplier (or counter width) is the next knob to tune per cache size, rather than any further layout work.
