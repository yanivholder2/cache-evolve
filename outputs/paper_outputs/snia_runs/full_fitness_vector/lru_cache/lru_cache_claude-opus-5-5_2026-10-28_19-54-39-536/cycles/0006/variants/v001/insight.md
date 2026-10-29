# Variant v001 Insight

## Helped

- per-op-time (put p50): The median put recovered most of the parent's regression. Two of the four SipHash computations on an evicting put, both from `hash_one`, became one Fx multiply each. `Occupied` puts now skip ghost work entirely, since it runs only in the `Vacant` arm. The map's own hashing of the new key (`entry`) and the victim (`remove`) is still there, so put now costs roughly two SipHashes plus two cheap multiplies. That is close to v002's two-hash baseline.
- hit-rate: The policy is unchanged, so hit rate held at the parent's level. The one small move is at hm_0/16K, where the Fx hasher with top-bit slots produces a different set of tag collisions and therefore different spurious reprieves. web_0/16K is unchanged. Both 131K points are identical because the ghost is disabled there. This confirms that a 16-bit fingerprint does not need a high-quality hash.
- per-op-time (get p99): Get p99 dropped. The protected-hit branch of `on_hit` now does a single OR in `set_protected(true)` and no AND. The tail of `get` has moved around between cycles before, though, so treat this as weak evidence.

## Hurt

- per-op-time (get p50, the summary): Get p50 got worse instead of recovering, even though the protected-hit path is now one OR shorter. The promotion branch also gained an explicit `prev &= !SC_BIT`, so the instruction count on `get` is roughly what it was. The parent insight blamed `set_protected`'s extra AND for the get p50 regression, and this result refutes that. The cause probably lies elsewhere: `put` was restructured and now does inline reprieve and victim work, which can change how the shared `on_hit`/`unlink`/`push_back` code is inlined and laid out, and gets also compete with the ghost table for cache.
- per-op-time (put p99): Put p99 rose even as p50 fell. The common evicting put got cheaper, but the slow tail did not. In that tail the reprieve still does a relink, the ghost slot read and write are random accesses into a table of up to 64 KB, and `map.remove(&old_key)` still runs a full SipHash. The ghost work also now sits inside the `Vacant` arm, with the victim selected twice (once for the ghost tag, once for eviction), which may make the code generated for this arm worse.

## Takeaway

Swapping SipHash for a one-multiply Fx hasher for the ghost tags, and running ghost work only on the vacant path, is enough to recover most of the median put cost without any measurable loss of hit rate. Fingerprint quality is not the bottleneck. However, removing the AND from `set_protected(true)` did not bring get p50 back, so the get-side regression is not caused by flag-bit arithmetic in `on_hit`. A follow-up should look at code layout and inlining of the shared list helpers, or at the ghost table's cache footprint, rather than shaving bit operations further.
