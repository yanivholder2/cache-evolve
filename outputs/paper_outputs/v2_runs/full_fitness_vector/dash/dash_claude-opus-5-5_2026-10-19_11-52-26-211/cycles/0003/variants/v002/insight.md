# Variant v002 Insight

## Helped

- per-op-time: In `promotion_get`, a stash hit on a non-hottest item now just calls `get_from_position`, an in-place LRU touch. The old path ran `remove_at` + `promote_to_target` + a possible `demote_from_normal_bucket` cascade, and that path now runs only on the second hit. Removing those cascades is where most of the `get` tail drop comes from, and it accounts for the small headline `get` p50 gain. `put` is unchanged within noise, as expected, since `DashSegment::put` was not touched.

## Hurt

- hit-rate: The second-hit filter cost more reuse than it saved. Under H02, the hit rate fell below the parent's 0.702 bar, so the idea is refuted. In a 2-slot stash bucket, a fresh key on its first hit (sitting at the Front/LRU end) now stays in the stash instead of moving to the target. There it competes with demotees entering at `DEMOTE_TO_STASH_POS = Back` and with later arrivals. So keys that have already been re-read once get evicted from the tiny probationary bucket before their second access. Under the parent, those keys would have been safe in the target tier.

(metadata-size did not change: the gate is a `const bool` and a position comparison. No fields were added, and `Item` stays at 40 bytes.)

## Takeaway

On this workload, after v003's Front-insert for fresh puts, one stash re-read is already enough evidence of reuse. Asking for a second hit at the stash MRU end filters out real reuse, not just one-off noise. That is because 2-slot stash buckets are too small to hold first-hit items until they are read again. Cycle 2's v002 showed the opposite direction (hotter promotion) also hurts, so the current single-hit, Front-of-target promotion looks close to a local optimum for stash→target admission. Still, the in-place touch shows that the remaining `get` tail latency comes from the promote+cascade path. Cutting it without hurting hit rate needs a cheaper cascade, not fewer promotions.
