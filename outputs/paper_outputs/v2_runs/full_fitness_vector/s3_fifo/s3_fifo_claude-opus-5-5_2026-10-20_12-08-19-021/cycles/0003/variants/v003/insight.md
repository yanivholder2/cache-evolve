# Variant v003 Insight

## Helped

- per-op-time (`put`): Removing the resize branches made the miss path shorter. The ghost-hit path no longer runs the `small_size -= 1` block with its fix-up `evict_small_inner(false)` and extra `evict_main()`. Small eviction no longer checks `grow_small` or runs a follow-up `evict_main()`. Each miss now does at most one main eviction pass, which fits the lower `put` p50 and the flatter tail.
- per-op-time (`get`): `get` improved by a similar amount, but its code is byte-for-byte the same as the parent's: one `index` lookup, one slab access, and a saturating freq bump. So this change cannot be credited for the `get` gain. It probably comes from benchmark variance or from a different resident-set or hash-table state under the new policy. The `get` gain should be re-measured before anyone relies on it.

## Hurt

- hit-rate: The paper's static configuration came in slightly below the adaptive parent on workload-a. Two things likely cost hits. First, the graduation rule is now `freq >= 1` and resets freq to 0, so one-touch small entries flood main and arrive with no second-chance credit. In the parent, only entries with two or more touches graduated, and they kept their accumulated freq. Second, a fixed 10% small queue gives new arrivals very little time to prove reuse, while the parent could grow small up to `capacity/2` when main filled with graduates.
- metadata-size: The summary did not move because `Entry<K,V>` and the `S3Fifo` fields are unchanged. The fields `small_size`, `main_size`, and `capacity` are kept but are now effectively constants. The ghost does grow: `ghost_cap = main_size` is now 90% of capacity, up from the parent's initial 75%. That means more live ghost keys and stamp records, and the dimension cannot see this cost.

## Takeaway

On workload-a, the published static S3-FIFO setup (10/90 split, `freq >= 1` graduation with freq reset) does not match the adaptive split. So the parent's adaptive gain was not only an escape from a bad 25/75 starting point: the adaptation, or the stricter two-touch graduation it was paired with, is earning a small but real margin. This variant changed the split and the graduation rule together, so it cannot say which lever cost the hit-rate. The next step should combine the fixed 10/90 split with the parent's `freq > 1` graduation rule (no freq reset) to separate the two.
