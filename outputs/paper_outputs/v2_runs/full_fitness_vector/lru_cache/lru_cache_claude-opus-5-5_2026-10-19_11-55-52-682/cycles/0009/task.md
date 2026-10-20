# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's Fx-hashed S3-FIFO. v001 widened the small queue, v002 ported the packed `u32` side array with key swap, and v003 made the ghost 2-way associative and twice as large. All three are listed as Winners, but none advanced hit-rate. v002 is the only real gain: a smaller per-node footprint with the same eviction order, paid for with slower `get`. v001 and v003 both lost a little hit-rate and slowed `put`.

## Patterns

- On workload-a, S3-FIFO's admission knobs appear to be at or near a local optimum, as SIEVE's in-queue knobs were in cycle 0003. A longer probation window took space from main, and extra ghost recall pushed long-reuse keys into main. Both changes cost more than they recovered. Main capacity for proven-reuse keys is the scarce resource.
- The bounded reach of the direct-mapped ghost was acting as a useful filter. Collisions limited how far back a "returning" key could be and still skip probation. Ghost tuning should shorten reach, not lengthen it.
- The side array's latency effect depends on the policy. On SIEVE it helped the `put` tail. On S3-FIFO's heavier miss path, that benefit disappears, and `get` now touches two cache lines (flags and value) instead of one. Representation changes still preserve eviction order safely, but they are no longer free on latency.
- The `put` path on S3-FIFO is now dominated by hashing and ghost probes on each miss. Changes that add ghost work show up at the `put` tail right away.
- Metadata accounting is still blind to heap side structures: the larger ghost and the `links` buffer and `Vec` header are not counted. The per-node wins overstate the real savings.

## Open questions

- Can VISITED stay inline with the value while only the links move out, keeping most of the memory win without the `get` penalty?
- Would a smaller ghost, sized to the small queue or to main's reuse distance, or a softer admission for ghost hits (e.g. entering main unvisited behind the hand) beat the cycle-0005 configuration?
- Does any of this hold at another cache size or trace? This has been unverified for five cycles, and the lineage is now tuning against a single point.

## Continuation

Keep cycle 0005 v003 as the hit-rate parent. Use cycle 0006 v002 only where memory matters, and try a variant that keeps VISITED next to the value. Drop wider small queues and larger or more associative ghosts. If ghost tuning continues, go smaller or make ghost admission less aggressive, and test split points just around 10% (1/8, 1/12) rather than wider. Before any further knob tuning, evaluate at a second cache size or trace, and count ghost and side-array heap bytes when judging metadata.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
