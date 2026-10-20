# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's SIEVE and each tried one fix. v001 dropped the back-link (singly-linked queue, hand carries its predecessor). v002 replaced the visited bit with a 2-level counter. v003 capped the sweep length. All three are listed as Winners, but only v001 improved anything: a smaller node with the same eviction order and no speed change. v002 lost a little hit-rate and got slower. v003 lost a little hit-rate and did not move the put tail.

## Patterns

- Changing the representation while keeping the policy identical is the only safe way to gain on this lineage. It improved metadata without risk. Node compaction stopped helping per-op-time once the hit path no longer relinks, so on SIEVE it is a metadata lever only.
- Varying how the hand handles queued nodes has run out of value on workload-a. More credit per hit (v002) and less patience per sweep (v003) both cost hit-rate. SIEVE's single bit with quick demotion of new keys looks like a local optimum for in-queue reuse signals.
- This cycle refuted cycle 0002's diagnosis of the put tail. Capping the sweep did not lower put p99, so long sweeps are too rare to set it. The likely cost is the per-miss HashMap remove/insert, key clone and cold-node touches. GCLOCK's tail cost does reappear when credit is deepened (v002), but that is a separate issue from SIEVE's baseline tail.
- Long runs of visited nodes do occur at size 2048 and they carry real reuse. Evicting them early hurts.

## Open questions

- Would history of evicted keys (ghost entries, an S3-FIFO-style small/main/ghost split, or ARC/LIRS-style non-resident tracking) add hit-rate where in-queue counters could not?
- Can the per-miss map work be reduced, for example with a single entry-API remove-and-insert, by reusing the hash, or by avoiding the key clone? Would that lower put p99?
- Does SIEVE's lead hold at other sizes or traces? That has been unverified for two cycles.

## Continuation

Use v001 (singly-linked SIEVE) as the parent. Drop further counter-depth and sweep-cap tweaks to SIEVE. For hit-rate, try a signal from outside the queue: ghost/history of evicted keys, or an S3-FIFO-like small probationary FIFO with a ghost queue. For put p99, target the HashMap path on misses instead of the hand. Include a second cache size or trace this cycle before building further on workload-a at 2048.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
