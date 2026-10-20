# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried: a second halving of the ghost table (v001), in-place reuse of the evicted node's slot in `put` (v002), and ghost-hit keys inserted at probation MRU or second position (v003). All three were Winners, but none showed a verified gain. v001 held hit-rate level, v002 improved nothing, and v003 lowered hit-rate. Winner status looks like noise tolerance.

## Patterns

- The ghost table still has headroom. A second halving did not cost hit-rate, but put tail latency did not improve either, so shrinking it no longer buys anything measurable.
- Cutting the allocate/free pair in `put` (v002) did not help. Map remove and insert dominate put cost, so node allocation is not the lever.
- A middle path for ghost hits (v003) hurt hit-rate. The ghost filter helps by keeping suspicious keys cheap to evict, not by rewarding remembered ones. Both ghost-hit promotion routes (protected last cycle, probation MRU now) have failed.
- `get` timing moved in all three variants, mostly at the tail, although `get` was never edited. Run-to-run or layout noise swamps small per-op-time deltas, as in every cycle since 0005.
- The metadata-size metric cannot see heap-allocated ghost memory, so the memory saving from ghost shrinking remains unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- What is the per-op-time noise floor? Without a repeat-run measure, no latency claim can be credited or blamed.
- Does the ghost-gated design generalize to non-scan workloads? This has been pending for many cycles.
- Can map operations per put be reduced structurally, for example with a single combined lookup/insert (entry-style API) or a cheaper map?
- Is the remaining hit-rate gain in admission or sizing, since insertion-position and segment-share levers are exhausted?

## Continuation

Treat cycle-0008's ghost-gated baseline, possibly with the smaller ghost table, as the parent. Drop ghost-hit insertion position, ghost shrinking, node-slot reuse, and protected-share steps. Next cycle, build a noise-floor and heap-inclusive memory measurement, and evaluate on non-scan workloads. Then try a structurally different lever: fewer map operations per put, or an admission policy.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
