# Variant v002 Insight

## Helped

- per-op-time: The CLOCK sweep clears reference bits as it passes items, so buckets no longer saturate the way the parent's permanent protection did — the parent's eviction scan often found no `lfu_counter == 0` and walked the whole bucket, while the self-clearing sweep usually hits an early cleared slot, shortening the put-path scan.

## Hurt

- hit-rate: Replacing permanent protection with a self-clearing second chance destroyed the scan resistance that was the source of v002's gain — `clock_victim_index` demotes a referenced item (`lfu_counter = 1 → 0`) on the very next sweep that passes it, so long-resident-but-hot entries lose immunity almost immediately and get evicted, collapsing behavior back toward plain LRU on the LIRS-style trace.

## Takeaway

The hypothesis's premise was inverted: on this trace, permanent protection was the *feature*, not the bug — cold-but-once-hot "squatters" weren't what capped hit rate, and giving protection a second chance sheds it too fast (one passing sweep clears the bit) to hold the working set. Second-chance aging trades away real hit-rate for slightly cheaper evictions, a bad bargain here.
