# Cycle 0002 Insight

## Cohort outcome

This cycle validated cycle 0001's open threads: the composite variant (v001) merged all three prior wins plus lazy ghost deletion and improved all three fitness dimensions simultaneously, while the two focused variants (v002, v003) confirmed each isolated change is independently additive. All three variants won.

## Patterns

- **The last O(n) hotspot is gone**: Replacing `ghost.retain()` with lazy tombstoning — removing the key only from the HashSet on ghost hit, deferring deque cleanup to the eviction capacity guard — cut put tail latency by 5× in the surgical variant and was the dominant gain in the composite as well. Amortizing per-hit cleanup across future evictions is a consistently powerful pattern here.
- **Orthogonality held at composition**: Queue sizing (1/10 fraction), parallel freq deques, and O(1) ghost operations all touched disjoint code paths in cycle 0001 and continued to stack cleanly when combined in v001 — no interaction penalty was observed.
- **Queue sizing introduces a put trade-off**: v003 confirmed that enlarging the main queue by shrinking small from 1/4 to 1/10 accelerates reads (shorter small-queue scan, more hot-set residency) but nudges put latency upward because the second-chance loop in `evict_main()` must traverse a longer queue to find a zero-frequency victim. This is the first observed cross-dimension tension from the sizing change.
- **Lazy deletion leaves deque slack**: Ghost tombstones may temporarily accumulate in the deque between capacity-guard sweeps. Under the current policy the deque can transiently exceed the live-entry bound; the guard drains them in future evictions, but the backlog is uncharacterized.

## Open questions

- Is the put latency increase from a larger main queue (observed in v003) masked in v001 by the O(1) ghost-hit path, or does it persist at higher ghost-hit rates?
- How large does the tombstone backlog in the ghost deque grow under workloads with dense ghost hits before the capacity guard catches up?
- The `evict_main()` second-chance scan is now the most expensive remaining loop — are there sub-linear victim selection strategies (e.g., approximate frequency indexing, segmented clock) worth probing?

## Continuation

v001 (composite: 1/10 small queue + parallel freq deques + O(1) ghost + lazy deletion) should be the new baseline for all further cycles; it dominates on every axis and eliminates all known O(n) paths. The next hypotheses should focus on the `evict_main()` second-chance scan, which is now the primary loop cost, and on characterizing whether the 1/10 small-queue fraction is at or near the optimal point for Zipfian workloads — particularly whether further shrinking it worsens cold-miss handling or puts pressure on the tombstone backlog.
