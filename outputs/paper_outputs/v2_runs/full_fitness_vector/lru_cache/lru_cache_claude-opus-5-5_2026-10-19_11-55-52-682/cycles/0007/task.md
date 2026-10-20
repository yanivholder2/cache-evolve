# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from singly-linked SIEVE (cycle 0003 v001). v001 grafted a ghost table of evicted keys onto the single queue. v002 swapped SipHash for an Fx-style hasher with `u32` map values. v003 moved the link and visited bit into a packed `u32` side array. All three are listed as Winners, but the outcomes split. v002 is the clear gain: faster get and put, hit-rate unchanged, and a smaller cache struct. v003 is a metadata-only gain with the same eviction order. v001 regressed on hit-rate and put latency.

## Patterns

- Representation-only changes stayed safe for the third cycle running. Both v002 and v003 kept eviction order bit-for-bit. The only policy change, v001, cost hit-rate.
- This cycle found the latency lever: hashing. Cycle 0003 already showed node layout no longer moves speed. v002 then moved get p50 for the first time in several cycles, and put p99 improved even more. This supports cycle 0003's view that the put tail comes from map work on the miss path, not from the hand. v003's denser sweep was roughly neutral on speed, which confirms that layout is now only a memory lever.
- A ghost signal needs a probationary queue to bypass. On a single SIEVE queue, a ghost hit can only extend the stay of keys whose reuse distance already exceeds the queue. Those keys push out better residents. v001 also added hashes on every miss, so the "outside-the-queue signal" question is still open. Only the minimal graft has been refuted.
- The metadata metric is drifting out of step with the real footprint. v001 put its growth in a heap table, and v003 put its policy state in a side array the per-node summary doesn't count. Read node-size wins with care unless side arrays are added in per entry.

## Open questions

- Does a real S3-FIFO split (small probationary FIFO + main SIEVE + ghost) gain hit-rate where the single-queue graft lost it?
- How much of put p99 remains after cutting probes per miss (entry-style insert, reusing one hash across the probe, remove and insert)?
- Do v002's hasher and v003's side array stack cleanly, and does the extra memory touch on each hit matter once hashing is cheap?
- SIEVE's lead at other cache sizes or traces is still unverified after three cycles.

## Continuation

Use v002 (Fx-hashed SIEVE) as the parent and fold in v003's side-array layout as a low-risk combination. For latency, cut the number of map probes on the miss path. Do not do further node-shape work for speed. Drop the single-queue ghost graft. If ghosts come back, they must come with a small probationary queue (S3-FIFO-style). Count side-array bytes per entry when judging metadata, and finally add a second cache size or trace.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
