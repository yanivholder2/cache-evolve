# Variant v002 Insight

## Helped

- per-op-time: Replacing the per-hit key-matched `realign_reuse` rebuild with an index-preserving `rotate_left(1)` applied identically to `items` and `reuse` removed the O(n²) linear-key-search step, collapsing both the get and put p99 tails toward the p50 — exactly the recovery v001's insight predicted.

## Hurt

- metadata-size: This variant's overlay carries only `associative_cache_bucket.rs` and drops the parent's slimmed `item.rs`, so `Item<u64,u64>` reverts from the 16-byte floor back to the 40-byte base layout — a regression unrelated to the policy change, since the reuse bit still lives on the (unmeasured) bucket.

## Takeaway

Gating move-to-back behind the reuse bit turned the quadratic per-hit rebuild into an O(bucket_size) rotate, which is the real prize: scan-resistance state can be maintained without any p99 blow-up. But the two-hit probation gate did not push hit-rate past the 0.7267 incumbent on workload-a — the same resident set survives whether promotion fires on the first or second touch — so the change bought latency, not accuracy, and silently gave back the parent's 16-byte Item by not re-applying its `item.rs` slimming.
