# Variant v001 Insight

## Helped

- hit-rate: Hit rate went above both the parent and the baseline. Each physical bucket now has one ClassicLRU order over all `2 * bucket_size` slots. That removes v003's split-LRU rules (timestamp-gated demotion, always dropping an evicted guest, and the promotion swap). So the item evicted is always the least recently used in the whole 32-way set, and a hot logical bucket can borrow its neighbour's slots without the extra churn. This supports v003's insight, which guessed that the half bookkeeping, not the pairing, was holding hit rate back.
- per-op-time (put p50/p99, get p99): Put got faster at both percentiles, and get's tail tightened. Three changes probably explain this. A miss now costs one scan and one `remove(candidate)` + `push`, with no demotion. The partner path is gone (remove from partner, evict from home, `insert_half_at`, re-push), and that path was the expensive tail case in v003. ClassicLRU's touch no longer calls `Instant::now()`.

## Hurt

- per-op-time (get p50, the summary value): The typical get got slower, so H01's claim that it would hold v003's get p50 failed. In v003 most hits were found by scanning a home half of about 16 items, and the touch only rotated inside that half. In v001, `find` scans the full 32-item bucket in insertion/LRU order. Then `rotate_left` shifts everything from the hit position to the end of the full bucket. On a typical hit, roughly twice as many 40-byte `Item`s are compared and moved. That costs more than removing the clock read saves.

## Takeaway

On this trace, one true LRU per merged 32-way bucket beats v003's split-half policy on hit rate, and it also removes the costly partner-promotion tail. So v001 is the right clean control for later policy experiments. The cost moved to the common get path: scanning and rotating twice as many items in one bucket raises the median lookup. The next lever is to make 32-way lookup cheap without going back to halves, for example with a compact key/tag array scanned before touching full `Item`s, or an LRU touch that doesn't move the whole tail. Also note that `Item` stays the same size because ClassicLRU no longer reads its `timestamp` field. That field is now dead weight under this policy and a candidate for shrinking metadata-size.
