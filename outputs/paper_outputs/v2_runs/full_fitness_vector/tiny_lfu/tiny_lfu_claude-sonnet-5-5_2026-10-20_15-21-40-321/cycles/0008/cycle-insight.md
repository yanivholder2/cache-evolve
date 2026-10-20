# Cycle 0008 Insight

## Cohort outcome

Three changes were tried: halving the sketch table (v001), shrinking the window to a smaller share of capacity (v002), and a compact Node with u32 slab-index links and packed flags, without segmentation (v003). All three were kept as Winners, but none improved speed or hit-rate. Only v001 and v003 gave a real footprint reduction, and each paid for it with slower get and put.

## Patterns

- Footprint reductions did not turn into speed. Both size cuts, one in the sketch and one in the Node, left latency worse. The working set seems small enough that locality gains are absent. The index-conversion and flag-unpacking overhead is paid on every operation.
- v003 undercuts the cycle-0007 lesson. The compact Node alone gave no speed gain, so cycle 0007's speedup came from something else, or was noise. The "footprint and locality pay" thread is now weak.
- Window size is not a hit-rate lever. A smaller window lost a little hit-rate and slowed put, since more inserts go through admission and eviction. This closes the window-sizing question from cycle 0007.
- Hit-rate has stayed flat or slightly down again across the cohort. Sketch, window, segmentation and Node layout have all failed to move it on workload-a.
- v002 showed unexplained growth in Node and struct size that the change should not cause, so its overlay may carry an unintended edit. Its attributions are inferred.
- Latency regressions are consistent in direction across v001 and v003, which makes them somewhat more believable than earlier noise-level deltas. Winner status here reflects the selection rule, not real gains.

## Open questions

- Where does time go in get and put? There is still no profile, and every blind hot-path guess so far has been flat or worse.
- Is the admission decision itself (compare incoming frequency against victim frequency) the only remaining hit-rate lever, or is workload-a at its ceiling for this design?
- Are the incumbents at the floor for this architecture, so that further gains need a different data structure?

## Continuation

Drop sketch size, window size, and Node compaction as standalone levers. They are all explored. Do not build on v002 without auditing its diff. Next cycle should either obtain a profile-driven target (map lookup, relinking or sketch estimate) before proposing speed changes, or try a structurally different admission or data-structure idea, such as an adaptive window or a cheaper hash map. Each hypothesis should name the specific cost it expects to move and be checked with repeat runs.
