# Variant v001 Insight

## Helped

- per-op-time: Reusing the winning candidate's already-computed frequency (no separate `estimate` call after the best-of-2 pick) kept the 8th-eviction path at 2–3 estimate calls, preventing the latency regression that cost cycle/0006/v001 on a 4-call path.

## Hurt

## Takeaway

Layering best-of-2 victim selection onto the 1-in-8 strict-gate path produced no measurable hit-rate gain over the parent's simpler always-pick-head approach. Both the LRU head and the second-oldest node are already cold relative to the working set, so choosing between them by frequency doesn't change which entries survive — the strict admission gate comparison against the incoming key is the operative filter, not which cold candidate is evicted. The change confirms that best-of-2 selection at the cold end of the LRU queue is not a lever for hit-rate improvement on this workload.
