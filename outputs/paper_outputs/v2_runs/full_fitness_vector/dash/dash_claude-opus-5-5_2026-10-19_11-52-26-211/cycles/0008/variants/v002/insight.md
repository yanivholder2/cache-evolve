# Variant v002 Insight

## Helped

- hit-rate: hit rate passed the hypothesis's "confirmed" threshold and landed just short of the full-rescue lineage's best. `stash_insert_with_rescue` now keeps a stash victim whenever its target bucket has a hole. Before, every victim of a full `s0`/`s1` pair left the cache. So almost all of rescue's hit-rate gain comes from target holes, and almost none from probing holes. This answers the hypothesis's isolation question.

## Hurt

- per-op-time (get p50): `get` p50 crossed the hypothesis's "refuted" threshold of ≥80 ns and reached the rescue-lineage range, so the latency half of H02 failed. Rescue never writes into a probing bucket, so `foreign` and the probing gate are unchanged. The `get` code path is also byte-identical to the parent. The cost therefore comes from fuller target buckets alone. Target scans have more residents to walk, and stash→target promotions now hit full targets more often. That triggers `demote_from_normal_bucket` cascades, and each cascade now ends with an extra hash plus a rescue insert.
- per-op-time (get p99): the `get` tail rose too. This fits the slow path getting longer. With targets full more often, a stash hit's `promote_to_target` more often runs the whole chain: evict, then demote, then stash insert, then rescue.
- per-op-time (put p50 / p99): `put` got slower, and its tail rose clearly. Every fresh put that overflows both stash buckets now pays for `get_index` on the victim, an `is_full` check, and often a `put_normal_known` write. Note: the parent's large `put` p99 drop was flagged as possible variance. Part of this rise may be that drop reverting, not new cost.

(metadata-size did not change. Rescue adds no per-bucket state, and `Item` and `Dash` are the same size.)

## Takeaway

Target-only rescue gets almost all of full rescue's hit-rate. But it pays almost all of full rescue's `get` latency, even though the `foreign`-gated probing scan stays skipped. So the latency comes from the higher occupancy in target buckets that rescue causes, not from writing foreign items into probing buckets. Any cheaper rescue has to limit how full target buckets get, or how many promotion cascades that fullness causes. Keeping probing buckets native-only does not recover the latency.
