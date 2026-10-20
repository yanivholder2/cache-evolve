# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a hit-path lookup collapse (v001, named the Winner) and a probation/protected segmentation of the recency list that targets scan-heavy traffic (v003). v002 produced no insight file and was rejected for infrastructure reasons, so it tells us nothing about its approach. Only v003 reports a hit-rate movement, and it is not a Winner.

## Patterns

- The Winner's hit-rate equals its parent on workload-a, which fits a change meant to preserve eviction and recency order. Its stated target, per-op time on the hit path, was not measured, so the Winner rests on behaviour preservation alone. This is the same gap carried from cycles 0001 through 0003.
- v003 is the only variant this cycle with a reported hit-rate effect, but its insight gives no direction or magnitude, and it is not in the Winner list. Whether that reflects a regression, a smaller gain, or a selection-criterion problem cannot be read from the insights.
- v003's insight attributes its mechanism to H03. Cycle 0002 framed H03 as a policy-neutral per-op check, not a segmentation change. Either the hypothesis mapping or the mechanism label is off, and the lineage should be checked before reusing either.
- v003 adds a second list pointer and demotion logic, so it likely carries per-op and size costs that have not been measured.

## Open questions

- Does v003's hit-rate effect on workload-a go up or down against its parent, and by how much? Its insight does not say.
- Why was v001 chosen as the Winner over v003 if v003 is the only variant that moved hit-rate?
- What are the per-op time and metadata size costs of the probation/protected split, and does the hit-path collapse in v001 reduce per-op time at all?

## Continuation

Next cycle should measure per-op time and metadata size on v001 and on v003 before either lineage is extended, since no cohort has produced evidence on those dimensions. Re-check the selection criterion so that v003's hit-rate result is weighed on its own, and confirm the H03 mapping. Keep the segmentation idea alive as the one hit-rate-moving candidate, and treat v001's hit-path collapse as a cheap, eviction-neutral candidate that still needs a per-op measurement.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a probation/protected split of the recency list (v001), a second-chance referenced-bit eviction in place of move-to-front (v002), and insertion of new keys at the LRU tail with only periodic MRU insertion (v003). Only v003 reports a hit-rate gain; v001 and v002 both report a hit-rate dip, yet v002 is listed as the Winner.

## Patterns

- The only hit-rate improvement came from changing where new keys enter the recency order (v003), not from adding structure. It is a scan-resistance effect that needs no second list or ghost set.
- Both changes that touch eviction order lowered hit-rate on workload-a at size 2048: the segmented recency list (v001) and the second-chance approximation (v002). Strict recency ordering appears to be doing useful work on this trace.
- The Winner list includes v002, but its own insight reports a hit-rate dip below its parent and no offsetting gain. Its selection rests on something the insight does not show, the same selection-criterion problem flagged in cycles 0003 and 0004.
- Per-op time and metadata size were not recorded for any variant this cycle. That gap now spans five cycles, so no lineage has evidence on the dimensions its changes were written to improve.

## Open questions

- What justifies v002 as a Winner when its hit-rate fell below its parent and no time or size gain was recorded?
- Does the LRU-tail insertion in v003 hold up on other traces, or is the gain specific to workload-a at size 2048? Its scan-resistance mechanism would only matter on scan-heavy traffic.
- Is the segmentation idea dead as written, or does the protected cap or demotion rule (the parameters v001 points to) carry real headroom?

## Continuation

Pursue v003's insertion-position change as the lead candidate: it is the only hit-rate gain this cycle and adds no structure. Drop the probation/protected split as written and drop second-chance eviction, since both lowered hit-rate. Re-check v002's Winner status before extending its lineage, and add per-op time and metadata size to the fitness vector before judging any lineage.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
