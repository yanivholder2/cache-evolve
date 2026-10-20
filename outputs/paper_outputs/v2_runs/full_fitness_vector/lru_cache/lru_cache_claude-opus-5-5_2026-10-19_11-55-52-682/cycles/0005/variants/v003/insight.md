# Variant v003 Insight

## Helped

- hit-rate: Adding a probationary FIFO lets one-touch keys from workload-a leave after a short stay in the small queue instead of a full SIEVE sweep of the whole cache. Main space is then kept for keys that proved reuse. The ghost now has something to bypass, so a returning key skips probation instead of pushing out a resident. The gain is real but modest and only just clears the hypothesis bar. That suggests workload-a has only a limited share of one-hit scan traffic for the split to filter.
- per-op-time: The hypothesis expected `put` to get worse, because of the second `hash_one` per miss and the random ghost read and write. Instead `put` held at p50 and improved at the tail. A likely reason is that most evictions now pop the small-queue head in O(1). Before, the main hand often had to walk past many set `visited` bits, so the long-sweep cases that drive p99 happen less often. `get` is functionally unchanged (one bit write, no relink), so its small movement is best read as noise or layout side effects, not as an effect of the policy.

## Hurt

- metadata-size: The summary per-`Node` size held, as designed: the `small: bool` fits in existing padding next to `visited`, and the size assertion still passes. The `LruCache` struct itself grew, though. It now carries the small-queue `s_head`/`s_tail`/`s_len`, `small_target`, the ghost `Vec` header and its mask. The ghost's heap buffer (one `u32` per slot, rounded up to a power of two of capacity) is a per-capacity cost that no current dimension measures.

## Takeaway

The question left open in cycle 4 is answered: a fingerprint ghost improves hit-rate once there is a probationary queue to bypass. Splitting SIEVE into a 10% small FIFO and a 90% main SIEVE gives a hit-rate gain on workload-a without costing per-op latency. The cheap O(1) small-queue evictions appear to more than pay for the extra hash and the ghost probe. The size of the gain points to the split ratio (`capacity / 10`) and the ghost size as the next things to tune. The unmeasured ghost heap should be tracked if metadata-size is meant to stay honest.
