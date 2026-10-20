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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried tail insertion of new keys with a sparse MRU insert, stacked on the parent's lookup collapse (v001), a stricter variant that promotes only on the second hit (v002), and a swap of the key index's hasher with eviction and recency left alone (v003). v001 is the only Winner, yet its own insight reports a hit-rate drop against its parent, and no variant recorded a per-op time measurement.

## Patterns

- Both tail-insertion variants (v001, v002) lost hit-rate on workload-a at size 2048. Tail insertion is the only mechanism this cohort tested that touches eviction order, and both of its forms moved hit-rate down.
- This does not match cycle 0005, where tail insertion was the only hit-rate gain. The same mechanism now reads as a loss, so the earlier gain either depends on the parent it was measured against or does not transfer to a stacked form. The cycles are not reconciled.
- v001 is named the Winner while its only measured dimension fell. The selection-criterion problem flagged in cycles 0003 through 0005 is still open.
- v003 (hasher swap) held hit-rate, as a placement-only change should, but per-op time was not recorded, so its one intended effect is untested. This is the same unmeasured per-op gap carried since cycle 0001.
- The H03 label now refers to a hasher swap here, a pre-sized index in cycle 0003, a per-op check in cycle 0002, and a segmentation change in cycle 0004. The hypothesis mapping is not stable across cycles and should be fixed before lineages cite it.

## Open questions

- Does tail insertion help on its own against the current parent? The cycle 0005 gain and this cycle's losses cannot be separated from the insights.
- Why is v001 the Winner when its hit-rate fell and no per-op or size gain was recorded?
- Does the hasher swap in v003 reduce per-op time at all?

## Continuation

Test tail insertion alone against the current parent before extending any stacked lineage, since the evidence for it now conflicts. Drop second-hit promotion (v002) as written. Re-check v001's Winner status before building on it, and record per-op time for v003 before judging the hasher swap. Pin down what H03 means so future cycles cite one definition.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
