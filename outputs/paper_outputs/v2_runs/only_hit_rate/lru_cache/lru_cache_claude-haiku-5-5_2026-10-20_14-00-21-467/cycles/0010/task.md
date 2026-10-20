# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried a ghost-set admission filter (v001), tail insertion of new keys (v002), and a frequency-aware eviction that picks the least-hit entry within a bounded four-entry tail window (v003). The Winner, v003, is the only variant that reports a hit-rate movement, and that movement is marginal.

## Patterns

- Only the frequency-aware tail-window eviction (v003) moved hit-rate, and it moved it slightly upward on workload-a at size 2048. Its gain is the one positive signal in the cohort, and its Winner status is consistent with that.
- The ghost-set admission filter (v001) left hit-rate unchanged. Its insight notes the fitness vector has no admission or rejection counts, so it cannot distinguish a filter that never engaged from one that engaged without effect.
- Tail insertion (v002) gave a hit-rate exactly equal to baseline. That is either a true null or a sign the insert-position change never affected evictions. This cycle's result conflicts with cycle 0005 (gain) and cycle 0006 (loss), so the tail-insertion evidence is not yet usable.
- v003 adds a per-entry hit counter whose memory cost is not in the fitness vector. Per-op time and metadata size were again not recorded, so the Winner's cost side is unknown. This is the same unmeasured gap carried since cycle 0001.

## Open questions

- Does the v003 gain survive on a second trace or a different cache size, or is it specific to workload-a at size 2048?
- Did the v002 tail-insertion overlay actually engage? Until a counter shows it did, its null cannot be read as evidence against the idea.
- Did the v001 admission counter ever reach the filter? If not, the ghost-set idea has not been tested at all.
- What do the per-entry counter and the tail-window scan cost in per-op time and metadata?

## Continuation

Pursue the bounded frequency-aware eviction (v003) as the lead lineage, but only after recording per-op time and counter memory and re-running it on a second trace or size. Verify engagement counters for the tail-insertion overlay (v002) and the ghost filter (v001) before ruling either idea in or out. Drop the ghost-set admission rule as written unless its counter shows it engaged.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried decaying the hit counters examined by the victim scan (v001), capping the per-entry hit counter at a small saturating width (v002, the Winner), and a count-min frequency gate on admission against the LRU tail (v003). None of the three reports a hit-rate gain; the Winner v002 is within noise of its parent, and its claimed benefit is a smaller per-node footprint that the fitness vector does not measure.

## Patterns

- No variant moved hit-rate upward. The two counter-based changes (v001, v002) and the admission gate (v003) all left workload-a at size 2048 flat or slightly down, so frequency signal did not translate into retained hits in this cohort.
- The Winner, v002, was selected for a footprint benefit that was never recorded. The same selection-criterion gap flagged in cycles 0003 through 0007 applies here again.
- None of the three insights confirms that its mechanism engaged. The overlay source could not be inspected, and v003's admission counts are absent, so flat results cannot yet be read as evidence against the ideas.
- Metadata size and per-op time were again missing from the fitness vectors. This is the same unmeasured gap carried since cycle 0001, and it blocks judging any change aimed at memory or time.
- The frequency signal itself keeps recurring across cycles (the bounded hit-window in cycle 0007, the counter changes here). Its effect on hit-rate has been small or null each time, so the open question is whether it helps on any trace beyond workload-a.

## Open questions

- Does the v002 counter cap reduce metadata size at all, and by how much? Without that number, the Winner has no measured benefit.
- Did the v003 admission gate ever reject a candidate? If it never engaged, the count-min idea remains untested.
- Does the v001 decay change hit-rate on a second trace or a size sweep, or is any effect hidden at this single point?

## Continuation

Re-check v002's Winner status before extending it: keep it only if metadata size is recorded and shows a real reduction with hit-rate held. Drop the count-min admission gate as written unless an engagement counter shows it rejected candidates. Pause the decay idea (v001) until it is tested on a second trace or size; do not build further on it from this cycle alone.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried widening the frequency-aware tail scan (v001), a correlated-reference filter on counted hits (v002, the Winner), and linking new keys at a mid-recency marker instead of the MRU head (v003). The only hit-rate gain reported this cycle belongs to v003, which is not the Winner; the Winner v002 shows no measurable change against its parent.

## Patterns

- Insertion depth is the only lever this cycle with a reported hit-rate gain. Placing fills partway down the recency order (v003) reduced displacement of the hot head on workload-a at size 2048. Its insight does not confirm the marker was engaged, so the gain is consistent with the mechanism but not yet attributed to it.
- Earlier cycles tested tail insertion with conflicting results (gain in 0005, loss in 0006, null in 0007). The midpoint form is a different position from MRU-tail insertion, so the conflict may be about insertion depth rather than the idea as a whole. This is a hypothesis to test, not a finding.
- Widening the frequency-aware victim window past four entries (v001) lost hit-rate. This matches the bounded-window result in cycle 0007: frequency bias helps only near the LRU end, and deeper scans evict entries the trace re-references.
- The Winner v002 has no recorded gain, and its own insight cannot confirm the filter landed. The selection-criterion gap flagged since cycle 0003 recurs: the only measured improvement sits in a non-Winner.
- Metadata size and per-op time are again absent from every fitness vector. This is the same gap carried since cycle 0001.

## Open questions

- Does the midpoint-insertion gain (v003) hold on a second trace or another cache size, and does it change when the marker sits at other depths?
- Did the v003 marker actually receive insertions, and did the v002 correlated-reference filter ever reject a hit?
- What does the midpoint marker cost in metadata and per-op time?

## Continuation

Pursue insertion depth as the lead lineage: build on v003's midpoint marker and test a small sweep of marker positions on a second trace before stacking anything on it. Drop widening the frequency-aware tail window beyond four (v001). Do not extend v002 as the Winner until it records a measured effect; verify its filter engaged before ruling the idea in or out.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
