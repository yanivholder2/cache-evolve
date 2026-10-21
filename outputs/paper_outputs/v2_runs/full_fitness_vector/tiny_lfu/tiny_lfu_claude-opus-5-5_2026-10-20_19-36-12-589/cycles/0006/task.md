# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

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


### Cycle 0004

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Two variants branched from cycle 0004's v003 (`u16`/2× sketch). v001 merged in the fused increment, and v002 removed the double count on get-miss followed by put. v003 branched from cycle 0004's v001 and added a ~1% plain-LRU admission window, with the region flag packed into a link bit. v002 gave the only hit-rate gain, and it was small. v001 merged cleanly but carried over none of the latency gain. v003 lowered hit rate and raised per-op cost. All three are listed as Winners, but only v002 moved a headline axis forward.

## Patterns

- The fused-increment latency gain did not carry over to the `u16` sketch. Hit rate was identical bit for bit, so the merge is behaviourally sound. That makes cycle 0004/v001's speedup look like either a 32-bit-counter effect or bench drift. Sketch micro-optimisation is used up, just as sketch accuracy was last cycle.
- How accesses are counted barely matters. The +2 bias on a miss applied to residents and newcomers alike, so it mostly cancelled in the duel. Fixing it only changes close duels. Together with cycle 0003's tie-rule result, this suggests the duel's inputs are now about as fair as they can get.
- A tiny window ahead of the duel hurts on workload-a. At ~1% it is too small to hold recency bursts. It also moves the duel from the arriving key to an older window head, which reorders admission badly. Every hit then pays for list selection and masked links, and every full miss re-hashes stored keys. This matches cycle 0002's aging result: the workload rewards stable frequency, and recency-oriented structure doesn't pay for itself.
- Packing the region flag into a link bit costs nothing per entry. If structural changes come back later, per-node bytes are no longer the obstacle.
- Per-op readings are still unreliable. v002 slowed down with no plausible cause in the code. Only v003's regression has a clear mechanism behind it.

## Open questions

- Would a larger window, around 5–20%, with cached per-node hashes or estimates so the overflow duel doesn't re-hash, behave differently? Or is any window wrong for this trace?
- With sketch and admission inputs both used up, is the remaining cost in `HashMap` lookup, `key_hash`, and relinking on every hit, rather than in the sketch?
- Is workload-a's hit rate now close to its ceiling for this design family?

## Continuation

Use v002 (single count per access) as the base. Optionally fold in v001's merged increment: it is orthogonal and leaves hit rate unchanged. Re-measure its latency side by side against cycle 0004's v003 before treating the regression as real. Drop sketch micro-optimisation, access-accounting tweaks, and tiny (~1%) windows. If you test a window again, make it substantially larger and cache each node's hash so the duel does no re-hashing. Otherwise, move the effort to the costs outside the sketch: map lookup, hashing, and relinking on every hit.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
