# Variant v002 Insight

## Helped

- per-op-time: Put got faster at both p50 and p99. On a full ClassicLRU bucket, the miss path no longer runs `evict_item()` (a dynamic `eviction_candidate_index` call, a full-bucket `items.remove(0)` memmove, and the same on `tags`) followed by `insert((n-1)/4)` (another memmove of about 3/4 of the bucket). It now does one `ptr::copy` of `items[1..=p]` plus a write at `p`, and a matching `tags.copy_within`. `len` never changes, and the victim-index check happens once at construction through `victim_is_front`. That cuts the bytes moved per steady-state miss from about 1.75 buckets to about 1/4 bucket. The p50 gain is larger than the p99 gain, which fits the claim that most puts take this full-bucket path.
- hit-rate: Not a gain, but the invariant held. All four cells match the parent exactly. With `p = (n-1)/4` for LIP, or `p = n-1` for BIP, the fused rotation gives the same order as the parent's remove-front-then-insert. The new `fused_matches_reference_remove_insert` test checks this in lockstep against a reference model across bucket sizes 1–64, including the BIP branch.

## Hurt

- per-op-time: Get p99 went up even though this change doesn't touch the get path. `get_position_tagged` and the hit-path `get_from_position` are byte-for-byte the parent's. The rise therefore looks like run-to-run tail noise, or a code-layout/inlining side effect of the larger `put`, and not a cost of the rotation itself. Get p50, which is the dimension summary, didn't move.

## Takeaway

Measured on its own, without the atomic-clock regression that hid it in cycle 0004, the fused evict+insert rotation is a real put-path win. It confirms that the two memmoves in remove-then-insert were the main cost of a ClassicLRU miss on a full bucket. Replacing them with one partial shift of only the first quarter keeps exact LRU/LIP/BIP semantics. The remaining put cost on full buckets is now that quarter-length shift plus the `tags` mirror. The get p99 change should be re-measured before anyone attributes it to this diff.
