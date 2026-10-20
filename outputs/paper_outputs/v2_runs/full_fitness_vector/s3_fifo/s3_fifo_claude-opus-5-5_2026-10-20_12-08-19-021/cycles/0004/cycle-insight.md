# Cycle 0004 Insight

## Cohort outcome

All three variants used cycle 0003's v002 (the fingerprint ring) as their parent, and each was meant to change one lever. H01 halved the ghost ring. H02 froze the split at a static 10/90 with `freq > 1` graduation kept. H03 was a policy-neutral cleanup of the `put` path. All three are Winners, but only v003 (H03) clearly improved: `put` got a little faster and hit-rate was identical. v001 moved hit-rate barely above zero, and v002 lost on both hit-rate and latency, so both made the cut on rank in a small cohort.

## Patterns

- **The adaptive split has now held up four cycles in a row, and it soaks up changes to other levers.** Halving the ghost barely moved hit-rate, because the adaptation and its `capacity/16` / `capacity/2` clamps absorbed most of the change. Freezing the split lost hit-rate even with the incumbent graduation rule kept. Together with cycle 0003's static-split loss, this is the strongest evidence yet that the adaptation itself earns its margin and is not just an escape from a bad starting split.
- **"One lever" variants keep coupling ghost size to the split.** v002 tied `ghost_cap` to the new main size, so the ghost also grew. That repeats the confound from cycle 0003. Any change to the split has to pin ghost size on its own.
- **`put` gains come from cutting machine work, not hashing.** v003's gain traces to swapping the ring's modulo wrap for a compare and to inlining the eviction functions. Its premise of redundant hashing was a miscount: the parent already hashed and cloned as little as the refactor did. Count hashes and clones in the parent before you claim them as a lever.
- **`get` latency is noise.** All three variants left `get` byte-identical, and all three saw it move, mostly upward. Because the headline latency metric tracks `get`, a `put`-only refactor cannot improve the summary.
- **`metadata-size` still misses ghost heap.** It hid both v001's ghost shrink and v002's ghost growth.

## Open questions

- Where does `small_size` actually settle? Four cycles have asked for this, and no variant has reported it.
- Does a static 25/75 split (the parent's starting point), with the ghost pinned at 75%, match the adaptive split? That would isolate the adaptation from the split ratio and the ghost size.
- Would loosening or tightening the adaptation clamps turn ghost length back into a lever that matters?
- Is the tiny gain from a halved ghost real, or within noise?
- Would a single-probe miss path (a precomputed-hash or raw-entry index) give a real `put` gain beyond v003's?

## Continuation

Adopt v003 as the new parent: it keeps the incumbent policy with a faster `put`. Stop trying to shrink the ghost, and stop replacing the adaptation with any static split unless ghost size is pinned at the parent's value. Productive next levers are the adaptation's clamps and trigger strength, main-ghost reinsertion into `small` (still untested), and a true single-probe index for `put`. Every variant must report where `small_size` settles, and must not credit `get` latency changes when the `get` path is untouched.
