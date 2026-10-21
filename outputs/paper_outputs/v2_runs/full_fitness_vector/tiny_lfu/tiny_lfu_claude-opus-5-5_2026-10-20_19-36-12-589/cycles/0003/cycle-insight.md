# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's v002 (compact `u32` links). Each made one isolated change to admission or the sketch: v001 let newcomers win ties (`>=`), v002 put all of a key's counters into one cache-line block, and v003 switched to conservative update. Only v003 raised hit rate, slightly. That is the first hit-rate gain in the run. v001 and v002 lowered hit rate a little.

## Patterns

- The hit-rate ceiling comes from how accurate the estimates are, not from the tie rule. Making ties favour the newcomer hurt, so on workload-a the strict `>` rule was not over-rejecting. Reducing count-min overestimation helped, because it keeps cold newcomers and cold victims further apart in the admission contest.
- Anything that adds correlated collisions hurts admission. The blocked layout broke row independence and inflated estimates, which is the opposite of what conservative update does. Sketch accuracy is the lever, and it can move either way.
- The sketch already fits in cache at these capacities. Layout changes aimed at cache misses have nothing left to win, so the earlier "layout-only is safe" heuristic does not cover the sketch.
- Per-op-time readings stayed unreliable. v001 was faster on every op with no credible cause in the code, which matches the environmental drift suspected last cycle. Small latency differences between variants still can't be trusted.
- v003's gain cost some typical `get` latency from its two-pass increment. Only `put` gets that cost back, by reusing the returned estimate.

## Open questions

- Can the conservative-update increment be made single-pass, or kept in registers, so that `get` loses its extra cost?
- Now that conservative update limits the build-up of overestimates, would aging or a doorkeeper help where aging alone failed in cycle 0002?
- Would a wider or deeper sketch, with row independence kept, extend v003's accuracy gain for an acceptable metadata cost?

## Continuation

Use v003 (conservative update) as the new base, and first tighten its increment loop to recover `get` latency. Keep pursuing estimate accuracy: lower-collision sketch sizing, or a doorkeeper or aging paired with conservative update. Drop the tie-admission changes and the blocked or cache-line sketch layouts. Benchmark any latency claim side by side against the parent.
