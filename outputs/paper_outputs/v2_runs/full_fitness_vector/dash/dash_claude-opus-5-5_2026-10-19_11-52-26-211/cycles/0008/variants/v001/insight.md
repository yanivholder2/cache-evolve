# Variant v001 Insight

## Helped

- per-op-time (get p99): the get tail fell. The hashes this change removed all sit on the longest get path: a stash hit, then `promote_to_target` into a full target, then `demote_from_normal_bucket`, possibly a cascade, then `stash_insert_known`. That path dropped from roughly seven hashes to two, so the saving lands where that chain runs, which is the slow end of the distribution.
- hit-rate (held, as required): hit rate is identical to the parent. That confirms `index_from_hash(key_hash(k), n)` reproduces `get_index(k, n)` exactly, and that the passed-down `is_borrowed = stash_idx != s0` and `is_foreign` flags keep placement and victim choice unchanged, as a state-neutral refactor must.
- metadata-size (held, as required): `Item` is still 40 bytes. The single hash lives only in locals (`h`, `eh`, `ch`) and is passed as call arguments, never stored per item.

## Hurt

- per-op-time (get p50): the median get got slower, not faster. This **refutes** the hypothesis on its own stated criterion. The median get probably never reaches the stash-hit → demotion chain. In the parent, `promotion_get` already used just one hash for `s0` and one for `target_idx`. Folding those two into one saved too little to show against the bucket scans, and the larger `stash_hit` signature (7 arguments) and extra known-index helpers may have cost something in inlining or code layout.
- per-op-time (put p99): the put tail rose back above the parent's level, even though fresh puts now hash once and skip the `stash_indices` / `put_stash_at_index` rehash. The parent's insight had already flagged its unusually low put p99 as likely run-to-run variance. This looks like that variance reversing, not a cost the change added. put p50 was flat, which fits that reading.

## Takeaway

Hashing is not a material share of the median get in this lineage. Taking the stash-hit/demotion chain from about seven hashes to two only shortened the get tail, because that chain is a tail event, not the common case the hypothesis assumed. The next get p50 gains have to come from the bucket scans on the `s0` → target lookup, not from hashing. Keep the single-hash plumbing for its p99 benefit, but don't expect any more hash removal to move p50.
