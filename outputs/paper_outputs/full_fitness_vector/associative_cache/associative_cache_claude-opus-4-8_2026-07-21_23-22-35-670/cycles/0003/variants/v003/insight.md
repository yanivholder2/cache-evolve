# Variant v003 Insight

## Helped

- hit-rate: Overriding the concrete bucket's `put` to `items.insert(0, item)` admits miss items at the front (index 0, the eviction candidate) instead of the back, so never-reused scan items are evicted first while items hit before the next conflicting insert get promoted to the protected back via `get_from_position` — probation-by-position lifts hit-rate well above the parent without touching `Item`.
- per-op-time: The `get`/`put` p99 tails came down relative to the parent; front admission is a single `Vec::insert` on a tiny bucket, and the earlier eviction of dead scan items keeps buckets less churned, so the positional change carried no latency penalty.

## Hurt

## Takeaway

Scan-resistance can be bought purely from insertion geometry: flipping admission from MRU-back to probationary-front on the ClassicLRU bucket shielded hot items from scan bursts and raised hit-rate while the 16-byte `Item` (key+value, no flag or counter) stayed byte-for-byte identical to the parent. The 2Q/S3-FIFO probation idea needs no per-item state — only where a miss lands in the bucket.
