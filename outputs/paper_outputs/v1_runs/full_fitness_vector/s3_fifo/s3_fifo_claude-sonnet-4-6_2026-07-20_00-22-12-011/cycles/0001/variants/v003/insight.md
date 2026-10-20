# Variant v003 Insight

## Helped

- **metadata-size**: Removing `freq: u8` from `Entry<K,V>` eliminates the padding byte the compiler inserted to keep the struct aligned, shrinking each entry from 24 to 16 bytes — exactly the goal of the hypothesis.
- **per-op-time**: The smaller 16-byte entry packs more items per 64-byte cache line, which directly benefits the linear-scan path that `get()` and `put()` both use to locate keys across `small` and `main`, cutting p50 and p99 latencies for both operations.

## Hurt

- **metadata-size** (struct level): The `S3Fifo` control struct grew from 112 to 176 bytes because two new `VecDeque<u8>` headers (`small_freq`, `main_freq`) were added — a 64-byte fixed overhead not captured by the summary metric, which tracks only entry size.

## Takeaway

Splitting the frequency counter out of `Entry<K,V>` into parallel sibling deques achieved the targeted 33% entry-size reduction and delivered an unexpected per-operation latency win by improving cache-line utilization during linear scans — indicating that entry struct size is a meaningful lever for this scan-heavy eviction policy. The cost is a one-time 64-byte growth in the `S3Fifo` control struct, which is amortized to negligible per-entry overhead at any practical cache capacity.
