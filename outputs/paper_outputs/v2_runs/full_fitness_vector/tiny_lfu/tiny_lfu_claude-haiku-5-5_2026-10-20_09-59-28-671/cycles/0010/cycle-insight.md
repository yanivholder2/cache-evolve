# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes: skipping redundant counter stores on the put path (v001), narrowing sketch counters to 8 bits while doubling columns (v002), and short-circuiting the get hit path when the node is already at the segment head, with a single hash per get (v003). All three are labelled Winners, but none shows a clear hit-rate gain, and the timing signals are single-run.

## Patterns

- No variant raised hit rate. v001 and v003 left it unchanged, and v002 dropped it slightly. The narrow-counter direction from Cycle 0009 did not carry over: the u8 layout here lost hit rate.
- The u8 variant may not have tested what it claimed. Its sketch came out at half the intended size, so the column doubling did not land as specified. Until the row and column counts are checked, this result says little about narrow counters in general.
- The two get- and put-path edits (v001, v003) are pure access-pattern changes. Their hit rate and metadata match the parent, which is consistent with eviction behaviour being left alone.
- Timing movements are weak. v001's put tail improvement and v003's put-side gain both come from one run, and v003 made no put-path change, so its put result is unexplained. v002's small latency drop was incidental to its aim.
- Metadata is flat in v001 and v003. v002 halved the sketch allocation, so its footprint change is real but was not the intended one.
- Source overlays could not be inspected, so mechanisms are inferred from the variant write-ups.

## Open questions

- Does the get-path MRU short-circuit in v003 hold its get gain across repeat runs, and what explains the put-side movement?
- Was the v002 sketch size shrink a sizing error, and does a correctly sized u8 sketch still lose hit rate?
- Does the single-loop put path in v001 keep its tail gain once the put median is measured over repeat runs?

## Continuation

Pursue the get-path short-circuit and single hash in v003 as the cleaner speed candidate, but confirm it with repeat get timings before building on it. Do not drop the narrow-counter direction on the strength of v002 alone: first fix its sketch sizing and re-run, since the Cycle 0009 hit-rate signal is still unresolved. Hold the put-path store-skip in v001 until repeat runs show a consistent put median gain.
