# Variant v003 Insight

## Helped

- per-op-time (put p99): The worst puts are misses. A miss scans the whole bucket to confirm the key is absent, then sweeps CLOCK. That full-bucket absence scan now runs over contiguous 8-byte keys instead of striding through 40-byte `Item`s. The 1024-wide buckets are where the dense mirror saves the most cache lines, so this tail is where the hypothesis's argument actually held.
- per-op-time (get p99): Long get scans, where the key sits deep in a wide bucket or is missing, also go through the dense `keys` array, so the tail improved a little. The gain is much smaller than on the put tail. This suggests deep scans are rare on the get path.

## Hurt

- per-op-time (get p50): A typical hit now touches two separate heap allocations instead of one. First it scans `keys` to find the index, then it loads `items[idx]` to read the value and set the reference bit. In v002 the scan ended on a cache line that already held the value. Each lookup also checks the `keys_dirty` branch first. When the scan is short, the cost of the second allocation outweighs the savings from density. The scan uses `position()`, which exits early, so it probably does not auto-vectorize as the hypothesis assumed. That last point is a guess: the eval artifacts do not show the generated code.
- per-op-time (put p50): A typical put pays the same split lookup. A replacement now writes to both `items[victim_idx]` and `keys[victim_idx]`, which are two cache lines in different allocations. The CLOCK sweep still strides through 40-byte `Item`s to read `lfu_counter`, so the mirror does nothing for the sweep.
- per-op-time (summary): The summary is get p50, the one number this hypothesis set out to cut, and it got worse.

## Takeaway

A key mirror stored as a separate `Vec` makes long, wide-bucket scans cheaper, which is why put p99 improved. But it adds a second memory stream to every hit, and on the common short-scan path that extra miss costs more than density saves. The scan, not the reorder, is the cost, but a separate key array is the wrong fix for p50. A follow-up should keep the key and the data needed on a hit in the same cache line. It could also use a small tag or fingerprint array that rejects most slots without loading `Item`s, and move the CLOCK reference bits into a dense side array as well, so that sweeps stop striding through 40-byte items. The extra 8 bytes per slot do not show up in the metadata-size types, so that dimension cannot reveal this memory cost.
