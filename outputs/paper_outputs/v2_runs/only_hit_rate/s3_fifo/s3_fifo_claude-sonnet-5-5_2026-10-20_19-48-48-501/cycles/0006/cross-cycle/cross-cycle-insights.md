## Recurring wins

- Admission-side sizing is the only lever that has paid off: a smaller small queue and a shorter ghost queue won repeatedly. Each step gave less than the one before.
- Capacity or ratio changes beat rule-level tweaks. The small/main split moved results more cleanly than any filter or aging change.
- Consolidating earlier winners (small-queue shrink, shorter ghost, ghosting main evictees) gave the latest gain. It added no per-key state.
- Adding ghost coverage without changing admission or protection rules is safe. At best it is marginally positive.

## Recurring failures

- Loosening admission loses (looser promotion, a longer ghost, pre-protected ghost re-admits). Weakly-reused keys in main displace useful residents.
- Aging or decay changes to main always lose (faster freq decay, a lower freq cap, in-place aging, halving). This workload rewards keeping reuse history.
- Rewriting main-eviction mechanics loses (lookahead, reinsertion caps). The unbounded second-chance scan does real protective work.
- Filtering ghost hits hurts, and so does giving them extra credit (a grace pass, a boosted freq, per-key ghost flags). Plain ghost-hit admission is already well used.
- Promotion-gate strictness and main insertion position had no measurable effect. They are weak or possibly no-op levers.
- Gains and losses are small, and the tied or neutral results are ambiguous because variant authors could not read the overlay source.

## Open questions

- Which part of the latest winner carries the gain: ghosting main evictees, or the re-bundled sizing?
- Do the gains hold on a second trace and cache size? All evidence so far is one trace at one size, so some effects may be noise.
- Does a very small probation window hurt scan-heavy traces?
- Were the tied variants actual no-ops? Future diffs should be checked to change behaviour.
- Why are metadata-size and per-op-time still unmeasured after six cycles? Does extra ghost work on `put` cost meaningful time?

## Per-policy notes

- S3-FIFO on the LIRS trace looks near a local optimum. Sizing is saturated, and eviction and aging rewrites regress.
- Current base: a very small small queue, a shorter ghost, and main evictees ghosted.
- Untried or under-tested directions: stricter promotion with a verified behaviour change, other traces and cache sizes, and an ablation of main-evictee ghosting.
- Cost dimensions (metadata, timing) have never been reported, so every cost claim is unverified.
