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
