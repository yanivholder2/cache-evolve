# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Each of the three variants made one isolated change to the baseline: v001 changed where promotions and demotions are inserted, v002 changed the normal/stash topology, and v003 added a frequency-aware LRU-tail victim choice. All three found real hit-rate headroom. The topology swap (v002, 24 normal / 8 stash) was the Winner because it improved hit-rate on both traces and both sizes, with no regression and no added mechanism.

## Patterns

- All three variants target the same weakness. The baseline's protected tier (target/probing) gives almost no protection, because promoted items are either evicted quickly (Front landing) or crowded out (small normal tier). Each variant fixed a different side of this, and none of them overlap.
- The variants help at different cache sizes. Fixing promotion placement (v001) helped most at the small size. Enlarging the protected tier (v002) helped most at the large size. The frequency filter (v003) helped at both sizes and helped hm_0 more. This suggests the fixes complement each other and are not redundant.
- hm_0 was consistently the more responsive trace, and the small size stayed hard for changes to topology alone.
- v001 showed that moving one lever shifts the dynamics of the others. MRU landing made the probing trigger fire too readily, and at the large size it penalised demoted items on web_0. Stacking these changes may produce similar interactions.
- No variant re-measured metadata-size or per-op-time. The cohort only shows effects on hit-rate.

## Open questions

- Does v002's topology combine additively with v001's MRU promotion and v003's frequency filter, or do they compete for the same gain?
- Is 3/4 normal the best split, or does a more extreme split (e.g. 7/8 normal) keep improving before the stash stops filtering scans?
- What does v003's eviction-window scan and bucket-wide counter halving cost in per-op-time?
- Would a stricter probing-promotion condition remove v001's large-cache web_0 regression?

## Continuation

Use v002's topology as the parent, and stack the other two levers on top of it one at a time: MRU promotion with a stricter probing trigger or a softer demotion position, and the frequency-aware tail victim with stash hits counted. Also sweep the normal/stash ratio past 3/4. Require per-op-time to be measured for any change on the eviction path. Drop further topology-only attempts at the small cache size, since the cache is too tight there for the split alone to help.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
