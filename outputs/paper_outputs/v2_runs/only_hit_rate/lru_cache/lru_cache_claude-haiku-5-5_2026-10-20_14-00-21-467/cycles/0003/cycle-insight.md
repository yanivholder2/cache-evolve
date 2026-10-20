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
