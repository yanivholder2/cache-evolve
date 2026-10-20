# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tried one representation change and two ghost-policy changes. v001 moved VISITED from cycle 0006 v002's `links` word into a dense bitset. v002 placed ghost returners at the SIEVE hand. v003 halved the direct-mapped ghost. All three are listed as Winners, but only v001 is a clear win: `get` and the `put` tail are faster and eviction order is unchanged. v003 is a near-noise hit-rate gain that comes with a free heap saving. v002 lost hit-rate.

## Patterns

- Both ghost probes this cycle point the same way: limit how far back the ghost reaches, but don't weaken what happens after a ghost hit. Returners placed at the hand were evicted before they could reuse. Shortening the ghost's reach cost nothing and may have helped slightly. A ghost hit is a strong reuse signal. Recall is the knob worth tuning, not admission placement.
- Flags that get written on every hit belong in a tiny, always-cached structure. The bitset recovered most of the side array's `get` penalty and made hand sweeps cheaper at the `put` tail. Each extra side array still adds fixed work on the miss path and grows the struct, so the median `put` rose slightly.
- S3-FIFO knobs on workload-a are now a flat plateau. A fourth straight cycle of admission and ghost tweaks produced only noise-sized moves in either direction.
- Latency attribution is getting unreliable. v002 slowed on a `get` path its change never touched, so environment drift is now as large as the effects being measured.
- Metadata accounting still misses heap side structures. v003's real ghost-buffer saving and v001's bitset don't show up in it.

## Open questions

- Should main evictions be recorded in the ghost? v002 found that returners evicted from main lose their fast path.
- Where does hit-rate start to fall as the ghost shrinks further, and does that point move with cache size?
- Can a single fused node-plus-flag layout give the bitset's `get` gain without an extra array header or extra work on every miss?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for six cycles.

## Continuation

Use v001's bitset layout combined with v003's half-size ghost as the new parent. Both keep eviction order safe, and they should stack. Drop placing returners at the hand, and stop sweeping small-queue or ghost size on workload-a. Before more policy tuning, add a second cache size or trace and repeat runs so latency deltas can be told apart from drift. The one policy probe still worth trying is recording main evictions in the ghost.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v001 (the VISITED bitset). v001 stacked the half-size ghost on top, v002 recorded main evictions in the ghost, and v003 moved VISITED into a spare bit of the map value. All three are listed as Winners. The two policy changes each gave a small hit-rate gain with no real memory or latency cost. The representation change won on the medians and on struct size but lost badly at the `put` tail.

## Patterns

- The half-size ghost gain held on a second node layout, so it is a real policy gain that does not depend on layout. Main-eviction ghosting was also positive, but barely. Together they close out ghost tuning on workload-a: what the ghost reaches back to and what feeds it now matter only at the margin.
- Where a flag lives has to match how it is accessed. The hit path reaches entries by key, but the SIEVE hand and small-queue pops reach them by index. Putting VISITED in the map bucket helped the readers that come in by key and made every hand step a hash plus a scattered probe. The `put` tail is set by the eviction scan, so index-addressed flag storage (the bitset) is still the better overall trade.
- The hit path is close to its floor. With the third structure gone, the gain was sub-nanosecond: one map probe plus one value line now accounts for nearly all of `get`.
- Latency measurement is still unreliable. v001's code left the `get` path untouched, yet its latencies all slowed by about the same proportion. Cross-run latency deltas cannot currently be attributed to a change.

## Open questions

- Do the half ghost and main-eviction ghosting stack, or do they interfere? A smaller table makes dilution from main victims worse.
- Can the bucket-flag `get` win and the bitset's cheap hand both be kept, for example by mirroring the flag or by having the hand sweep touch fewer nodes?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for seven cycles.

## Continuation

Use cycle 0008 v001 (bitset plus half ghost) as the parent. Optionally fold in v002's main-eviction ghosting as a single combined check, then stop probing the ghost. Drop map-bucket VISITED unless the hand gets index-addressed access back. Before more micro-tuning, put latency comparisons in the same run as the parent and evaluate at a second cache size or trace. Further hit-rate gains will probably need a structurally different idea, not more S3-FIFO knob changes.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants branched from cycle 0008 v002, which ghosts main evictions. v001 halved the ghost on top of it. v002 added a TinyLFU count-min duel at small-queue exit. v003 moved values into the map buckets. All three are listed as Winners, but only v001 is non-negative: a sliver of hit-rate gain with no cost. v002 lost on every axis. v003 left hit-rate unchanged and slowed both `get` and `put`.

## Patterns

- Ghost tuning is finished. The half ghost and main-eviction ghosting interfere rather than stack. Both make the ghost more selective, so combining them mostly adds collision dilution. The hypothesis's own exit condition is effectively met.
- The first structurally new admission signal, frequency, also failed to break the plateau. It reduced hit-rate because the duel ran the SIEVE hand speculatively and wore down main's VISITED protection. The sketch also undercounted hot main keys while overcounting churny probationary ones. The ghost already captures most of the returning-key signal on workload-a. This is more evidence that the plateau belongs to the trace, not to the admission policy.
- The hit path's floor is the map probe, not the value line. At this capacity the 16-byte arena stays cache-resident, so removing the `nodes[idx]` read saved nothing. Widening buckets made every probe, insert and remove slower. Map bucket width is now the latency lever, and it is better made smaller than larger.
- Metadata accounting is still blind to the heap, which made all three results look better than they are: the ghost table, the sketch, and the larger hash table all go uncounted. v003's apparent per-node halving only moved bytes into the map.

## Open questions

- Would frequency admission work if it only peeked at the victim, without clearing VISITED bits, and counted main hits on every hit? Or is workload-a simply unable to reward any admission signal?
- Can the map bucket shrink below key+`u32`, for example with a fingerprint-indexed or open-addressed table keyed into the arena?
- Does the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for eight cycles, and it is now the main threat to every conclusion above.

## Continuation

Make cycle 0009 v001 the parent, and treat ghost size and inputs as closed. Drop value-in-bucket layouts and drop the TinyLFU duel as specified. Revisit frequency only with a non-destructive victim peek and faithful hit counting, and only after it is shown to matter on a second trace or cache size. That evaluation should be the next cycle's priority over any further workload-a tuning. For latency, target narrower or cheaper map probes.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
