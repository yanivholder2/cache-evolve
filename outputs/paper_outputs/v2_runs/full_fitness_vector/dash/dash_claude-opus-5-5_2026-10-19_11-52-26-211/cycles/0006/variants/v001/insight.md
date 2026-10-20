# Variant v001 Insight

## Helped

- hit-rate: Rescue sends a stash-overflow victim into its target or probing bucket when either one has a free slot, instead of dropping it. Victims that would have left the cache now stay and can still be hit. The hit rate lands exactly on cycle-5 v002's level. This confirms the port is faithful. It also confirms that v002's lookup order (`s1` checked early) never affected which keys stay resident, because each key lives in only one slot.

## Hurt

- per-op-time (get p50 and p99): `promotion_get` has the same code as the parent, yet `get` p50 went up most of the way toward v002's level. The cost therefore comes from cache state, not from lookup order. Rescue keeps target and probing buckets fuller, so target/probing hits and every miss's `get_position` scan check more occupied slots. Rescued items also sit at the LRU end (`InsertPosition::Front`), the slot a scan reaches last.
- per-op-time (put p50 and p99): Both `put` percentiles went up clearly. Every overflowing stash insert now also computes `get_index`, checks `is_full` on one or two normal buckets, and does an extra `put_at` write. Demotion cascades run against fuller normal buckets, which lengthens the eviction chain behind each insert.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

The hypothesis's latency claim is mostly refuted. Rescue alone brings back most of v002's `get` slowdown even with `s1` probed last. So most of v002's latency came from normal buckets that rescue keeps fuller, and only a small part came from stash-first order. The hit-rate gain from rescue costs scan length on every `get` and work on every overflowing `put`. Cheapening that, for example with fingerprint or tag filtering inside buckets, matters more than any further change to the probe order.
