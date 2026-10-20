# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three attacks on segmentation's remaining cost were tried: a head-adjacent relink skip for protected hits (v001), a CLOCK-style second-chance reference bit replacing relink-on-get (v002), and removal of the hash avalanche fold (v003). All three were Winners. The real gains were v003's put speed and smaller struct, and v002's modest hit-rate gain with cheaper get. v001 was neutral on hit-rate and slower.

## Patterns

- The hash fold was a measurable cost on the segmented variant, mostly on put. A single multiply plus rotate gave enough mixing with no hit-rate change. This answers the cycle-0003 question: the fold, not the u32 conversion alone, was a major source of the slowdown.
- Cheaper promotion by skipping relinks only for nodes at or near the head did not pay off. Few hits land there, so the added branch cost more than it saved. Any promotion skip must cover a large share of hits, or add nothing to the hot path.
- Removing relink from get (v002) moved the work to eviction. Get got slightly faster, put tail got worse, and node size was unchanged. This is the same work-shifting pattern seen with lazy demotion in cycle 0003, though here the get hot path really did shrink.
- A single reference bit gave only a small scan-resistance gain, so it is real but limited.
- v003's struct-size drop was not targeted, which suggests the change touched more than the hash arithmetic. Attribution is inferred, since authors could not read diffs.

## Open questions

- Does the second-chance bit compose with the fold-free hasher, and do their gains add up?
- Can the eviction walk in v002 be bounded or amortized to cut the put p99 without losing the hit-rate gain?
- Why did put slow in v001, which was not its target? Possible causes are a shared touch path, code layout, or noise.
- Does the v003 struct shrink hold, and what caused it?

## Continuation

Combine v003's fold-free hasher with v002's reference-bit get path as the next baseline. Then bound the eviction scan, for example by capping the number of referenced nodes passed per put, to recover put p99. Drop head-adjacent relink-skip ideas and further hasher tweaks. Check hit-rate on workloads other than the scan-heavy trace.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three small refinements of the cycle-0004 baseline were tried: the fold-free hasher combined with the reference-bit get path (v001), a capped second-chance eviction walk (v002), and a larger protected share of the segmented cache (v003). All three were Winners, but the gains were small. Only v002 clearly changed the trade-off, and it paid in hit-rate.

## Patterns

- Combining the fold-free hasher with the reference-bit path (v001) preserved eviction order and left hit-rate and size unchanged. Speed moved only marginally and may be noise, so the two cycle-0004 gains compose weakly. The hash is now a small share of per-op cost.
- Capping the eviction walk (v002) cut get and put latency, including put p99, which was the cycle-0004 target. It did so by coarsening recency, which lowered hit-rate. Latency and hit-rate now trade off directly through the cap.
- Raising the protected share (v003) slightly lowered hit-rate on the scan-heavy trace. A smaller protected segment and a larger probationary one look at least as good, so the split is a weak lever.
- v003 edited only a construction-time constant, yet its latency shifted. This points to layout or measurement noise at the nanosecond scale, so small per-op-time deltas across the cohort should not be credited to the changes.
- Variant authors could not read the code, so all attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Can the put p99 gain be kept with a larger or adaptive cap, or with a cheaper rule for choosing which referenced nodes to spare, so hit-rate recovers?
- What is the noise floor of per-op-time, and which recent "speed wins" fall below it?
- Does a smaller protected share improve hit-rate beyond the baseline split?
- Does hit-rate behave the same on workloads other than the scan-heavy trace? That check has been pending for several cycles.

## Continuation

Treat v001's combination (fold-free hasher plus reference bit) as the baseline, and explore the cap in v002 as a latency/hit-rate dial, looking for a setting or an adaptive rule that keeps put p99 low without losing hit-rate. Probe a smaller protected share rather than a larger one. Drop further hasher and constant-tuning ideas that sit within noise. Measure a noise floor and evaluate on non-scan workloads before accepting small speed claims.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three small policy tweaks on the cycle-0005 v003 parent were tried: a larger protected share (v001), demoted nodes reinserted at the LRU end of probation (v002), and `put` on an existing key no longer promoting (v003). All three were Winners, but the changes were tiny. Only v001 gained anything real, a small hit-rate improvement.

## Patterns

- The protected-share trend keeps improving hit-rate across steps, but each step gains less. The split is a weak lever with diminishing returns.
- Demoting a protected node to the next-to-evict position hurt hit-rate slightly. Demoted nodes have shown reuse and are better treated as warm, not cold.
- Skipping promotion on `put` updates left hit-rate unchanged. The trace rarely overwrites resident keys, so hit-rate is decided by the `get` and miss-insert paths.
- Metadata size was flat across the cohort, as expected for edits that touch no layout.
- Per-op-time deltas appeared even where hot paths were untouched (a constant-only edit, an untouched `get`). These are noise, which confirms the cycle-0005 suspicion that small latency deltas are not attributable. Winner status here likely reflects noise tolerance, not improvement.
- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Where does the protected-share trend turn, if it does? Or is it saturating?
- Is there any hit-rate lever left in segment-boundary policy, or does the remaining gain lie elsewhere (admission, sizing, ghost history)?
- What is the actual per-op-time noise floor, and would a repeat-run measurement separate real deltas from noise?
- Does any of this hold on workloads other than the scan-heavy trace? This check has been pending for many cycles.

## Continuation

Drop `put`-update semantics, further demotion-position tweaks and more protected-share constant steps. They are exhausted or in the noise. Next, try a structurally different hit-rate lever, such as an admission filter or ghost-entry history for scan resistance, and use this cycle's v001 (larger protected share) as the baseline. Measure the noise floor and evaluate on non-scan workloads before crediting any speed claims.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
