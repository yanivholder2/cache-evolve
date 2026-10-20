# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's v003, the fast-hash version. v001 added Caffeine-style sketch aging with a 4-bit cap. v002 packed the list links into compact `u32` indices. v003 picked the victim as the coldest of a small sample of LRU-end entries. Only v002 produced a real gain, a halved per-entry `Node` size. No variant raised hit rate, and all three regressed on per-op time at least partly.

## Patterns

- Hit rate is still flat or worse. Aging (v001) and frequency-aware victim choice (v003) both tried to use the sketch more, and neither helped. Aging alone made hit rate worse: under the strict `>` admission rule with ties going to the victim, coarser counts mean more ties, so more good newcomers get rejected. Victim sampling changed nothing, because the entries near the LRU end have nearly equal estimates.
- The admission rule and the counter resolution interact. Changes to the frequency signal can't be judged on their own while the tie-break rule and the lack of a window or doorkeeper stay fixed.
- Any extra work in the hot path shows up in latency right away. Per-op data-dependent branches and bookkeeping (v001), and extra per-miss hashing (v003), each gave back latency that cycle 0001 had gained.
- Layout-only changes are the one reliable win. v002 shrank the metadata with no change in behaviour. Its latency regression has no credible cause in the code and looks like benchmark-environment noise.

## Open questions

- Is v002's per-op regression real? Benchmarking it side by side with cycle 0001's v003 on the same machine would settle this.
- Would aging help if paired with a tie-aware or randomised admission rule, or with a small admission window or doorkeeper, and with cheaper reset bookkeeping?
- Is workload-a's hit rate close to its ceiling for a TinyLFU-family design, so that efficiency is the only axis left to improve?

## Continuation

Make v002 (compact links) the new base, and first re-measure its latency against cycle 0001's v003. Drop LRU-end victim sampling. Don't re-submit aging on its own. If you revisit aging, pair it with a change to admission or tie handling and keep the bookkeeping out of the per-op hot path. Layout or efficiency changes that add no hot-path work remain the safest bets.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
