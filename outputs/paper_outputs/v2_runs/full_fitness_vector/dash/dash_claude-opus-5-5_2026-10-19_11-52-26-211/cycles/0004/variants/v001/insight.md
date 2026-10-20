# Variant v001 Insight

## Helped

- hit-rate (held, as intended): Hit rate matches the parent exactly. This confirms the refactor kept the mutation sequence unchanged. Passing `target_idx`/`probing_idx` into `promote_to_target`/`promote_to_probing` and computing the returned slot with `inserted_position` changes how the item is *found*, not where it goes. The `debug_assert_eq!` checks on both the passed-in index and the computed position were enough to land the change without a correctness slip.
- per-op-time (get p99 only, weak): `get` p99 dropped. That fits the change, because the work removed (one key re-hash plus one destination-bucket `get_position` scan) sits only on the stash-hit and hottest-target promotion paths, which are the slow tail of `get`. But `put` p99 also fell even though `put` was not touched in this diff. So part or all of this move may be run-to-run noise, not a real gain.

## Hurt

- per-op-time (get p50): The headline `get` p50 did not drop below the parent's. It ticked up slightly, so the hypothesis's speed claim is refuted. The median `get` is a target or probing hit, which never reaches the edited promotion branches. Even on promotions, one hash and a scan of a short bucket are small next to `remove_at` + `put_at` and any demotion cascade. The small rises in `get` p50 and `put` p50 (`put` is untouched) look like noise, not a cost of the change.

(metadata-size did not change: no fields were added, and `Item` stays at 40 bytes.)

## Takeaway

Removing the duplicate hash and the post-insert bucket re-scan from `promotion_get`'s promotion branches is provably state-neutral, but it does not move median `get` latency. On this lineage, the p50 is set by the non-promoting hit paths and by the cost of the promotion moves themselves, not by recomputing where the promoted item landed. Any further p50 gain has to come from the common target/probing lookup path or from doing fewer bucket moves. Trimming bookkeeping around promotions is not enough. The `inserted_position` invariant, that the cascade never writes back into the destination bucket, is now asserted and can be reused safely.
