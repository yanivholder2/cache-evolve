# Variant v002 Insight

## Helped

(none) Hit-rate is bit-identical to v003 in every trace/size cell. That is expected: `prefetch_hit_path` only issues hints, and the new `node_prefetch_hit_sequence_matches_parent` test checks list heads, tails, lengths and every node's links against a v003-order `get`. Metadata-size is also unchanged, because `prefetch_node` and `prefetch_hit_path` are methods and no struct changed. Get p50 did not move either.

## Hurt

- per-op-time (get p99): The tail the hypothesis set out to cut got worse instead. `prefetch_hit_path` starts with a demand load of `self.node(idx)` to read `region`, `prev` and `next`. So the neighbour and destination-tail prefetches cannot issue until the hit node's own miss resolves, which is the same dependency chain `on_hit` already had. The variant adds branches and up to five extra `prefetcht0` instructions to the slow promotion path, while the single sketch RMW that sits between them and `on_hit` hides very little of what remains of the neighbour misses.
- per-op-time (put p99): The update-in-place branch of `put` got the same prefetch block but no overlap. There, `increment_hashed` already runs before the `if let Some(idx)`, so the prefetches go out right before `node_mut(idx).value = value` and `on_hit`. With nothing to hide behind, the hint is pure added work, and the put tail regressed.
- per-op-time (put p50): A slight regression, consistent with the same extra hit-path instructions on update-in-place puts. Fresh inserts and evicting inserts were left untouched.

## Takeaway

Prefetching list neighbours only pays off if the addresses can be known before the node that holds them is loaded. Here they can't: `prev`/`next`/`region` live inside the hit node itself, so a prefetch sitting just one sketch RMW ahead of `on_hit` cannot break the pointer-chase. It only lengthens the path. This rejects the cheap version of the v003 diagnosis but not the diagnosis itself. If the get p99 cost really comes from promotion's scattered node touches, the fix has to remove the dependent loads or start them much earlier, for example by keeping link/region data out of line, next to the map value. Adding hints on top of the same chain does not help. For `put`, the hint was placed after the increment, which removed even the small overlap window `get` had.
