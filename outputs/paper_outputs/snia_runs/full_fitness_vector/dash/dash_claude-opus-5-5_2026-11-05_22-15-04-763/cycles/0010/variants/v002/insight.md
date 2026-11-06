# Variant v002 Insight

## Helped

- per-op-time (`get` p99): The read tail dropped clearly. This supports the claim, repeated since 0006, that the `get` tail sits on the second-stash-hit promotion chain. Moving that chain (stash removal, normal-tier MRU insert, victim demotion to stash MRU, `refresh_stash_mask`) into one `#[cold] #[inline(never)]` function, with each shift loop replaced by a single slice rotation, made the rare slow path cheaper. It also no longer inflates the hot `get` body.
- per-op-time (`get` p50): The median get moved only marginally in the right direction. That fits the i-cache argument: the hot body shrank to hash, mask test, bucket/stash scan, first-hit bump plus a call. But the effect is small enough to sit within run-to-run noise, so it does not show that layout was a meaningful p50 lever.

## Hurt

- per-op-time (`put` p99): The write tail regressed sharply, by more than `get` p99 gained. That repeats the 0006 v003 pattern the hypothesis warned about: changing the shared shift/demotion helpers had a layout and cost effect on `put`. The likely mechanism is that put's full-bucket insert and stash-demotion path now goes through the out-of-line cold function, or through the same rotate-based helpers. On `put` this path is common, not rare. Paying a non-inlined call into a cold text section, plus an out-of-line memmove for a small fixed-size bucket, lands on put's eviction tail. The parent's open-coded loop was cheaper there.
- per-op-time (`put` p50): Essentially flat with a hair of slowdown. This is consistent with the extra cost being confined to the eviction and demotion branch that only fires on full buckets.

(hit-rate and metadata-size did not move. That matches the hypothesis invariants: ordering after each op is identical to the parent, and no fields were added.)

## Takeaway

Marking the stash-promotion chain `#[cold]` is the right call for `get`, where the chain is rare, and it gets the targeted p99 win. The catch is that the same helpers, or the same chain, also do the full-bucket insert and demotion work that is routine for `put`. Labelling them cold for both callers moves the tail from reads to writes instead of removing it. The next step is to split the paths. Keep the cold out-of-line entry for `get`'s promotion only, and give `put` its own inlined (not duplicated-into-`get`) shift for bucket-full insertion. Alternatively, check whether `slice::rotate_right` on these short arrays compiles to a libc `memmove` call that costs more than the parent's unrolled loop.

(Note: I could open only the overlay's `src/shared/utils.rs`, which this change does not touch. I could not list or locate the segment/bucket source in this session. So the claim about how `put` reaches the cold function or the rotate helpers is inferred from the hypothesis and the fitness deltas, not checked against the code.)
