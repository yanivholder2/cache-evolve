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
