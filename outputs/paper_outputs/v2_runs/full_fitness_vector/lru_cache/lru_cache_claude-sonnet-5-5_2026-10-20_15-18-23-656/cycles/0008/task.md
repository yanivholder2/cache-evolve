# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two ghost-history variants and one micro-optimization were tried: ghost-hit keys sent straight to the protected segment (v001), cold keys inserted at the LRU end of probation unless the ghost table remembers them (v002), and an evicted-key clone removal in `put` (v003). All three were Winners, but only v002 produced a real gain, a hit-rate improvement paid for with slower put. v001 improved nothing, so its Winner status looks like noise tolerance, not merit.

## Patterns

- Ghost history works as a scan filter on insertion position, not as a promotion trigger. Demoting cold keys to the next-to-evict slot (v002) helped hit-rate. Promoting ghost hits into protected (v001) hurt it, because returning evictees are mostly not hot enough to displace protected entries.
- Both ghost designs cost the same on the miss and eviction path: an extra hash, a random table access and side memory. This showed up as worse put, especially at the tail. v002 left get unchanged. v001 slowed both.
- The ghost table lives outside `Node`, so the per-node size metric hides its memory cost. Metadata-size comparisons undercount this family of changes.
- v003 confirms again that key-copy micro-optimizations are invisible. For a plain integer key, the clone is free, and put cost is dominated by map and list work. Its small latency shifts are within noise.
- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Can the ghost cost be cut, for example by hashing only on eviction and reusing the hash already computed for the map, or by a smaller or sampled table, so put p99 recovers?
- Does a middle path work: ghost hits inserted at probation MRU, with no protected admission and no LRU-end insertion for cold keys?
- How much of v002's gain depends on the scan-heavy trace? Does it hurt or help on non-scan workloads?
- What is the real noise floor of per-op-time? This has been pending for several cycles.

## Continuation

Use v002 (ghost-gated LRU-end insertion) as the next baseline. Aim at cutting its put and memory overhead, such as a cheaper fingerprint, a smaller table, or a reused hash, and at checking hit-rate on non-scan workloads. Drop ghost-to-protected promotion (v001) and key-clone or other micro-optimizations that sit inside the noise.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
