# Variant v001 Insight

## Helped

- per-op-time: `get` p99 improved, and the per-op-time summary (`get` p50) did not. The likely reason is that the hash savings fall on the slow paths. These are second-stash-hit promotions through `promote_to_target_at` / `promote_to_probing_at` and their `put_at_idx` cascades, which used to hash `item.key` again through `get_index`. The common hit path only drops one hash.
- hit-rate: Hit rate is bit-identical on every trace and size, as the invariant requires. This confirms that `index_from_hash(hash_key(k), n)` reproduces `get_index` exactly, so the refactor changed cost and nothing else.

## Hurt

- per-op-time: `get` p50, the headline target, is flat, so the hypothesis's main claim fails. `put` p99 also got worse. `put` does not go through `promotion_get`, and this change only touched the shared `put_at_idx` / demotion helpers. So that regression is more likely noise or code-layout drift than a real cost.

## Takeaway

Removing one or two extra SipHash calls (`DefaultHasher`) per `get` did not lower the median. That rules out repeated hashing inside the segment as the cause of the `get` p50 gap, which is the falsifying outcome the hypothesis anticipated. The gap more likely comes from the stash-first probe itself, or from the second hash at the Dash level (`segment_index`, left alone here). The single-hash change is still worth keeping because it trims the stash-promotion tail at no hit-rate cost.
