# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried layout-level changes to the LRU node representation, aiming to shrink per-entry metadata without altering eviction order. Only v003 (index-width reduction using a sentinel instead of `Option<usize>`) produced a usable result and was named a Winner, but its evidence is incomplete: the size and per-op-time effects are unmeasured, and the other two variants produced no usable evidence.

## Patterns

- The one evaluated variant held hit-rate unchanged from its parent, which is consistent with a change that should not alter eviction order. This is the only cohort-level signal so far, and it does not show any size benefit.
- The Winner's credit rests on a dimension it was not measured on. Its fitness reflects hit-rate only, so the metadata-size goal went unscored.
- Two variants were rejected for infrastructure reasons, not strategy: one hit a session limit and recorded no tokens, the other produced no insight file. Nothing can be concluded about their approaches.

## Open questions

- Does the index-width reduction actually shrink per-entry metadata once measured, and does the sentinel check add per-op cost that offsets it?
- Would the other hypotheses in this cycle (H01–H03 broadly) have produced comparable hit-rate stability, or do they touch eviction order and therefore need a separate check?

## Continuation

Next cycle should re-evaluate v003's size and per-op-time dimensions before building further on it, and treat its size win as unconfirmed. Keep pursuing representation-only changes that preserve eviction order, and re-run the variants that were rejected for infrastructure reasons rather than dropping their strategies.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried one representation-level change (replacing optional links with a sentinel index) and two behaviour-level ideas: a ghost-based admission filter and pre-sized index allocation. Only v001 produced a usable result and was named the Winner, and its measured gain is on hit-rate, not on the node-size goal its change was written for.

## Patterns

- The Winner's change is a layout change that should preserve eviction order, and its hit-rate did not move from its parent. That is consistent with behaviour preservation, but it says nothing measured about metadata size.
- The same sentinel-index change already appears as cycle 0001's v003 credit, so the size claim is still carried forward without a measurement.
- The hypotheses differ in how much they touch eviction. H01 is a layout change that must keep hit-rate fixed; H02 deliberately changes what stays resident and must be judged on hit-rate and per-op cost; H03 leaves policy and layout alone and targets per-op time only.
- Per-op time has still never been measured on any lineage, so no cohort has yet produced evidence on that dimension.

## Open questions

- Lineage is ambiguous: the hypotheses describe v001's hit-rate as a gain over baseline, while the variant insight reports no movement against its parent. Which parent the gain is measured against needs to be pinned down before any comparison.
- H01 proposes re-applying a sentinel change that v001 already appears to contain. Is there a remaining size effect to find, or is H01 redundant?
- Does ghost-based admission (H02) improve hit-rate enough to justify its bookkeeping cost on the per-op dimension?

## Continuation

Next cycle should prioritise measuring per-op time and metadata size on v001 and the existing sentinel lineage, since neither has real evidence yet. Pursue H03 as a cheap, policy-neutral per-op check against the baseline, and treat H02 as the only hit-rate-changing bet worth running. Drop H01 unless it is reframed to measure size on the current winner rather than re-applying the same change.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes that each left eviction order alone: skipping relinks when a hit node is already the MRU head (v001), a ghost-based admission filter (v002), and pre-sizing the key index at construction (v003). Two variants are named Winners, but none of the three reports a hit-rate or per-op movement against its parent, so the Winner status rests on evidence the insights do not show.

## Patterns

- All three variants held hit-rate exactly equal to their parent on workload-a at size 2048. That fits the changes being behaviour-preserving, but it means the cohort has produced no measured gain on any dimension.
- No variant was scored on per-op time or metadata size, so the goals that motivated H01 and H03 remain untested. This is the same gap carried from cycles 0001 and 0002.
- v002's ghost-filter result may reflect the filter never engaging on this trace rather than a real null. Whether the admission path was reached is unknown.
- The Winner list includes v002, but its own insight reports no gain over its parent. The selection criterion needs checking before its lineage is built on.

## Open questions

- What justifies v002 as a Winner when its hit-rate matches its parent and its per-op cost was not recorded?
- Does the head-hit relink skip (v001) or the index pre-sizing (v003) reduce per-op time at all? Neither has been measured.
- Did the ghost-based admission filter (v002) ever run on the trace, and if not, what workload would exercise it?

## Continuation

Next cycle should add per-op time and metadata size to the fitness vector before judging any of these lineages, since that is the only way to test H01's and H03's central claims. Keep the head-hit relink skip and index pre-sizing as cheap, eviction-neutral candidates, and re-check v002's Winner status and whether its filter engaged before pursuing ghost-based admission further.


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
