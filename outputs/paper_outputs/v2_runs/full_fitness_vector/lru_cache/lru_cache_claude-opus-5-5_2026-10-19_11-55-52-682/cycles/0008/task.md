# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Following cycle 0004, this cohort combined the Fx hasher with the side-array layout (v001), replaced hashbrown with a hand-rolled key-less `u32` index (v002), and added the first real S3-FIFO split: a small FIFO, a main SIEVE and a fingerprint ghost (v003). All three are listed as Winners. v003 is the real advance, with the first hit-rate gain in several cycles and no latency cost. v001 is a mixed combination: a better put tail and a smaller struct, but a flat get tail. v002 kept the eviction order but regressed on latency, sharply at the put tail.

## Patterns

- The open question about signals from outside the queue is now settled: ghosts help when there is a probationary queue to bypass. The same ghost idea that lost in cycle 0004 as a single-queue graft gained here inside an S3-FIFO split. The gain is modest, which suggests workload-a has only limited one-hit scan traffic.
- Structural policy changes can also fix the tail. S3-FIFO's O(1) small-queue evictions improved put p99 even though each miss did more hashing and ghost work. That is the first policy change in this lineage to help latency, and it suggests long main-hand sweeps still matter at the tail some of the time.
- The hashing lever has mostly been used up on the hit path. Cheap hashing now mainly helps the miss path and struct size. Get latency is now set by how many separate memory locations a hit touches, which is why the side array and Fx did not stack on get.
- Hand-rolled indexes lose to hashbrown when they cannot reject a bucket without dereferencing the arena. Key-less buckets and backward-shift deletion that re-hashes through the arena made the clusters much more expensive at the put tail. The idea of using one hash per miss is not wrong, but it is only worth doing inside a table that filters buckets cheaply.
- Metadata accounting is still incomplete. The S3-FIFO ghost heap and the denser index of v002 are both invisible to the current dimensions.

## Open questions

- How much hit-rate is left in the S3-FIFO knobs, such as the small-queue ratio, ghost size, and whether main evictions should feed the ghost?
- Can the visited bit sit next to the data a hit already reads, so the hit path touches one location instead of two without growing the node?
- Can hashbrown's entry or raw-table API deliver the single-probe miss path without giving up its tag filtering?
- Does S3-FIFO's gain hold at another cache size or trace? This has now been unverified for four cycles, and it matters more now that a policy change has finally won.

## Continuation

Use v003 (Fx-hashed S3-FIFO) as the parent. Tune its small-queue ratio and ghost sizing, and fold in v001's representation gains (key swap on eviction and the zero-sized hasher) as a low-risk combination. Drop hand-rolled key-less hash tables. For latency, use hashbrown's own entry or raw APIs, or place the visited bit with the data a hit already reads, rather than doing more hasher work. Count the ghost heap when judging metadata, and add a second cache size or trace before tuning S3-FIFO further on workload-a alone.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
