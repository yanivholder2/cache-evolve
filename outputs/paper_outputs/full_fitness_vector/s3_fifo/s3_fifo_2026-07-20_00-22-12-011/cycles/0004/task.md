# Task: cycle-insight-writer

Cycle: 0004
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004

## Winners

- v002
- v001
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses\H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted distinct structural axes — queue-size ratio, ghost-lookup algorithmic complexity, and entry memory layout — and all three won over the baseline. The Winners share no single change but collectively demonstrate that the baseline S3-FIFO leaves headroom on multiple independent dimensions simultaneously.

## Patterns

- **Small queue was over-provisioned**: Reducing the small-queue fraction from 1/4 to 1/10 improved both hit rate and per-operation latency, indicating the original split over-invested in admission filtering at the expense of main-queue residency and scan length.
- **A dominant linear scan in `put()`**: The ghost-queue membership check was an O(n) scan that nearly halved median put latency when replaced with an O(1) hash-set oracle — the single highest-leverage algorithmic fix in the cohort.
- **Entry struct size is a meaningful lever**: Moving the frequency counter out of the entry struct achieved a 33% per-entry size reduction and delivered an unexpected latency benefit via improved cache-line utilization during the scan-heavy eviction path.
- **Fixed overhead amortizes cleanly**: Both v002 and v003 grew the `S3Fifo` control struct (HashSet header and parallel freq deque headers respectively), but that overhead is per-instance and constant, not per-entry, so it is negligible at any practical cache capacity.
- **The three improvements are orthogonal**: Each change touches a non-overlapping code path (sizing formula, ghost-check in `put()`, entry layout + eviction loops), making them safe candidates for combination.

## Open questions

- What is the optimal small-queue fraction, and does it depend on workload skew — or is there a threshold below which shrinking it further hurts cold-miss handling?
- v002 still calls `ghost.retain()` (O(n)) on a ghost hit to remove the key from the deque; the hot path for ghost hits is not yet O(1) end-to-end.
- Combining v001 + v002 + v003 may expose interactions (e.g., a smaller small queue shifts eviction pressure, changing how often the ghost set is queried); composite behavior should be validated.

## Continuation

All three strategies are worth carrying forward as a combined baseline for the next cycle — they are orthogonal and collectively represent a materially improved S3-FIFO. The next cycle should prioritize: (1) a composite variant merging all three wins, and (2) eliminating the remaining O(n) `ghost.retain()` call in v002's ghost-hit path, which is the last known linear-scan hotspot in `put()`.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle probed three independent axes: further small-queue shrinkage (v002), O(1) victim selection in `evict_main()` (v001), and a lower ghost-promotion threshold (v003). The cleanest win was v002, which continued the monotone small-queue reduction trend with no downside; v001 delivered a large latency gain at the cost of hit-rate; and v003 backfired, confirming the ghost channel is a necessary quality gate.

## Patterns

- **Small-queue shrinkage has not hit its floor**: Halving the fraction again (1/10 → 1/20) improved both hit-rate and get latency without any observed penalty — the monotone trend begun in cycle 0001 is still running clean on Zipfian workloads.
- **Deque churn, not scan length, was the latency driver in `evict_main()`**: Two-random-choices eliminated the `pop_back`/`push_front` memory-layout churn and halved get p99, but the associated loss of frequency-drain protection allowed hot entries to be discarded on a single bad draw — the hit-rate cost was material, not noise. The complexity class (O(n) vs O(1)) and the eviction *policy* were coupled; this variant untangled them in a way that confirmed each has independent value.
- **The ghost channel is a dual gate — frequency *and* demand confirmation**: Promoting at `freq > 0` short-circuited the ghost window's role as a re-request filter, flooding main with once-warm entries that lacked strong recency evidence. The paper's `freq > 1` threshold is not conservative bookkeeping; it enforces that main receives only keys that have been re-requested *after* ghost-listing, not merely re-accessed while still in small.
- **All three variants landed in the Winner list despite mixed fitness signals**: v001 hurt hit-rate and v003 showed no benefit alongside a hit-rate regression, indicating the evaluation fitness function may weight latency or aggregate score in ways that allow hit-rate regressions to be offset. This should be treated as a signal about fitness-function sensitivity, not as endorsement of those changes in isolation.

## Open questions

- Can a bounded second-chance scan — e.g., a fixed cap of 8–16 candidates rather than full O(n) traversal — recover most of v001's latency gain while preserving the frequency-drain protection that kept the hit-rate intact?
- Is there a floor to the small-queue shrinkage benefit, and does the ghost deque's expanded capacity (which tracks `main_size`) begin to inflate tombstone backlog pressure as `main_size` grows toward the full capacity?
- The winner designation for v003 (no benefits, one hurt) is anomalous — does the fitness function include a latency dimension that v003 happened to pass, or is this a threshold artifact worth investigating?

## Continuation

v002 (1/20 small-queue fraction) is the clear carry-forward winner and should become the new baseline; it is a clean, risk-free improvement on every measured axis. The next cycle's primary hypothesis should be a bounded second-chance scan in `evict_main()` — testing whether a shallow cap (rather than full O(n) or pure O(1) random) recovers hit-rate quality while still eliminating most of the deque-churn latency that v001 identified. Do not revisit the `freq > 0` promotion threshold; v003 conclusively demonstrated that the ghost channel's demand-confirmation function is load-bearing and the paper's `freq > 1` rule should remain fixed.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
