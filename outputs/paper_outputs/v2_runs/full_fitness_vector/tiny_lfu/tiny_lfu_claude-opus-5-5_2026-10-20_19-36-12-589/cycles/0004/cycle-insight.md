# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle 0003's v003 (conservative update), and each made one change to the sketch. v001 fused the increment into a single pass held in registers. v002 added Caffeine-style halving every 10×capacity increments. v003 swapped to `u16` counters at twice the width while keeping the sketch the same size in bytes. All three are listed as Winners, but they won for different reasons. v001 recovered per-op cost and left hit rate unchanged. v003 raised hit rate slightly at no metadata cost. v002 lowered hit rate, and its faster per-op times are most likely environmental noise, not a real gain.

## Patterns

- How conservative update is written now costs almost nothing. v001 showed that the earlier `get` penalty came from the two-pass reload loop, not from the update rule itself. What's left is within the drift of the bench.
- Sketch accuracy is giving less and less. Halving collision noise (v003) raised hit rate only a little. Once conservative update is in place, CountMin overestimation is close to used up as a lever.
- Decay hurts on workload-a no matter what counter it's paired with. Aging failed with 4-bit counters in cycle 0002 and failed again here with full-width counters and conservative update. Workload-a rewards stable long-term frequency. Under a strict duel against the LRU head with no window, halving removes the margin that keeps scan traffic out.
- Saturating `u16` counters cost nothing, so wider counters were wasted bytes. The bytes saved this way are better spent on width.
- Per-op readings are still unreliable. Two variants with no credible cause for a speedup (v002, v003) both got faster on every op. Only v001's latency gain has a mechanism in the code that explains it.

## Open questions

- Do v001's fused increment and v003's `u16`/2× width combine cleanly, giving both the latency and the hit-rate gain at once?
- Now that sketch accuracy has stopped giving gains, is the remaining hit-rate gap in the eviction or admission policy: the plain-LRU victim, the lack of an admission window or doorkeeper?
- Could aging work in a much milder form, with a far longer period or a sample that counts only misses rather than hits on resident keys? Or should it be dropped for this workload?

## Continuation

Merge v001's fused increment with v003's `u16`/2× width sketch as the next base, and confirm that both gains hold side by side against each parent. Drop blanket periodic halving, and stop tuning sketch width or overestimation. Look for hit-rate gains in policy structure instead, such as a small admission window or doorkeeper ahead of the duel. Keep per-hit and per-op cost low, because cycle 0001 showed that heavy list restructuring doesn't pay off.
