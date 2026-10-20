# Variant v001 Insight

## Helped

- hit-rate: Hit rate went up from the parent and lands exactly on the level the hypothesis gives for cycle-4 v002. The gain comes from the ported two-choice `stash_insert`. When `s0` is full, fresh puts and demotees now overflow into the neighbouring `s1` instead of evicting from `s0`, which absorbs stash-bucket conflicts. The exact match with v002 confirms that moving the `s1` probe to the end of `promotion_get` changes only the order of lookups. It does not change any mutation, because a key lives in at most one slot.
- per-op-time (get p50, relative to v002): Per the hypothesis, `get` p50 is now below v002's level. So the second stash scan that v002 put in front of every lookup was a real cost on target and probing hits. Moving `s1` behind target and probing, and computing it lazily via `stash_neighbor` only when a lookup would otherwise miss, takes most of that cost off the hit paths.

## Hurt

- per-op-time (get p50 and p99, relative to the parent): `get` is still slower than in the parent. Every miss now scans one more 2-slot stash bucket (`s1`) and computes `s1`. Rare `s1` hits pay for all four scans before they are promoted. Neither cost can be avoided while `s1` is probed unconditionally. The p99 rise fits the slow path getting longer: an `s1` hit means a full scan plus a stash→target promotion.
- per-op-time (put p99, weak): `put` p99 crept up. Every stash insert now goes through `stash_insert`, which computes `(s0, s1)` and can check `is_full` on two buckets. `put` p50 did not move, so this shift is small and may be noise.

(metadata-size did not change: no per-item fields were added, and `Item` stays at 40 bytes.)

## Takeaway

On this lineage the two-choice stash pays for itself in hit rate, and its lookup cost is split between hits and misses. Probing `s1` last recovers the part of v002's `get` regression that hits paid for, while hit rate stays identical. The remaining gap to the parent comes from misses and rare `s1` hits scanning one more bucket. To close it, the next step should skip the `s1` probe on most misses, for example with a per-`s0` "has overflowed" bit or a fingerprint filter. Reordering further cannot help, since `s1` is already last.
