# Cross-Cycle Insights

## Recurring wins

- Making one change per variant pays off. Bundled changes in cycles 1–2 hid each other's effects and kept the hm_0/web_0 trade-off in place. Isolated changes in cycle 3 removed that trade-off, and cycle 4's merge of isolated wins added up cleanly.
- Judging a key only by its reuse while in main works on both traces. Two filters do this: reuse-gated main-victim ghosting, and resetting freq/reused on graduation. Each won alone in cycle 3, together they won at every trace/size point in cycle 4, and neither added entry size or latency.
- Spending ghost budget selectively (filtering out unreused victims) lengthens useful history at no cost.
- Structural changes that leave hit rates bit-identical are the only reliable source of timing gains: the slab plus hash indexes (cycle 2) and the Fx-style hasher (cycle 3). Per-entry metadata stayed flat throughout.
- hm_0 reliably responds to how main retains keys: more main residency, main-victim history, or reuse-based credit.

## Recurring failures

- Fixed-ratio and region resizing (small-queue %, ghost region splits) only move gains between hm_0 and web_0 without a net win (cycles 1–2).
- Making the ghost larger hurts. A ghost bound beyond `main_size` lets stale reuse back into main, which pushes out live residents. Cycles 1, 2 and 4 all point to the ghost bound being coupled to other variables, and to the fix being selectivity rather than size.
- Loosening graduation to `freq >= 1` causes web_0's regression. The freq reset alone is harmless.
- ARC-style adaptive small/main splitting regressed when the signals were unbalanced and the ghost was tied to a fixed bound (cycle 1). It has not been retried.
- Policy-only diffs keep showing timing shifts their changes can't explain, in both directions, in every cycle. These are environmental noise and earn no timing credit.
- Cutting probe count by merging the resident and ghost indexes slowed things down, tails most of all. Higher load factor and wider buckets outweighed the fewer lookups.
- The metadata-size metric keeps missing real heap growth: ghost key storage, hash maps, stale ghost tuples, unified tables.

## Open questions

- What explains web_0's remaining loss at the large size? The ghost bound and ghost competition have been ruled out. The small-queue share (`capacity / 4`) is the last candidate and is still untested in isolation on the current parent.
- Can main-retention knobs (`MAX_FREQ`, graduation threshold) add hm_0 gains without the web_0 cost that `freq >= 1` showed?
- Would a unified index pay off with explicit load-factor headroom or a compact slot (a `u32` slab index plus a ghost flag bit)?
- Would a balanced adaptive split work on top of the current filtered-ghost parent?
- Timing needs a same-environment re-baseline. Should metadata reporting add heap bytes per resident key?

## Per-policy notes

- S3-FIFO (the only policy explored so far). The current best lineage is slab + hash, then Fx hasher, then both reuse filters with the `freq > 1` threshold and the ghost bound kept at `main_size` or lower (cycle 4 v001). Build on it.
- S3-FIFO on hm_0: sensitive to how main retains keys and to main-victim history, and gains the most from the reuse filters.
- S3-FIFO on web_0: sensitive to admitting freq-0 or loosely qualified keys into main and to squeezing the probation window. It regresses under a looser graduation threshold or a larger ghost, and its weakest point is the large size.
- S3-FIFO implementation: hashing and slab layout are where timing gains come from. Probe count matters less than table occupancy and bucket width.
