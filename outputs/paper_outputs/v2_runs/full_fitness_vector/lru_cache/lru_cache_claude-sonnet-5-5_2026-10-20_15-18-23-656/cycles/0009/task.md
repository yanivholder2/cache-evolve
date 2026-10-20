# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three low-risk refinements of the cycle-0007 ghost-gated baseline were tried: a halved ghost table (v001), a larger protected share of the segmented cache (v002), and fewer redundant hashes in `put` (v003, ghost lookup gated on a full cache and the victim hashed once). All three were Winners. The gains were small. The ghost-cost work reduced put tail latency, and the constant tweak gave a tiny hit-rate gain.

## Patterns

- Ghost overhead can be trimmed without losing the scan-filter benefit. A smaller table (v001) and less hashing (v003) both cut put tail latency while hit-rate held. The cycle-0007 ghost design was oversized and redundant.
- Both ghost-cost wins reach only the put tail. The median put is dominated by hash plus map insert, and `get` is untouched. Further gains need structural changes, such as fewer map operations per put, not fewer hashes in a fixed sequence.
- The protected-share trend still gives hit-rate gains, now on top of ghost-gated insertion, but they are tiny. It remains a weak, low-risk knob with diminishing returns.
- Per-op-time deltas again appeared on paths the change cannot touch (get after a ghost-only edit, get p50 after a constant-only edit). This is noise, so small timing deltas stay unattributable.
- The metadata-size metric is blind to heap-allocated ghost memory, so the claimed memory savings are unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- How far can the ghost table shrink, or become sampled, before hit-rate degrades? The current evidence only shows that halving is tolerable.
- Can the number of map operations per put be cut, to move the median put and not just the tail?
- Does the ghost-gated design, with its larger protected share, hold up on non-scan workloads? This check has been pending for many cycles.
- What is the per-op-time noise floor? Repeat-run measurement is still missing.

## Continuation

Treat the v001 and v003 combination (smaller ghost table plus single-hash gated eviction path) as the baseline. Pursue structural put-path reductions, such as fewer map operations, and add a heap-inclusive memory measure so ghost cost is visible. Drop further protected-share constant steps and hash or clone micro-optimizations, since they are saturated or within noise. Evaluate on non-scan workloads and measure a noise floor before crediting small speed claims.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
