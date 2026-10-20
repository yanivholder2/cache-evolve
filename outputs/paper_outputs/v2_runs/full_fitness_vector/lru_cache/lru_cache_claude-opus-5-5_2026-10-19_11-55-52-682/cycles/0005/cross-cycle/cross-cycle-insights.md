# Cross-Cycle Insights

## Recurring wins

- **Changes to representation alone have been the safest gains (cycles 1–5).** Narrowing links to `u32`, removing the back-link, packing state into side arrays, reusing padding bytes and swapping the Fx hasher all kept eviction order identical. Each one improved metadata or latency without costing hit-rate.
- **Moving away from strict LRU order pays off on hit-rate.** The approach that works: demote new keys quickly and promote reused keys lazily. It started with SLRU, CLOCK and BIP (cycle 1), then SIEVE (cycle 2), then S3-FIFO (cycle 5). Each step forward came from handling one-hit or scan keys better, not from deeper reuse counters.
- **A hit path that never relinks keeps get cheap.** Every hand-based policy (CLOCK, GCLOCK, SIEVE, S3-FIFO) kept get latency low. Once relinking was gone, get latency depended on how many separate memory locations a hit touches, not on node size.
- **The levers for latency moved over time.** First node layout (cycle 1), then hashing (cycle 4), then structural O(1) small-queue eviction (cycle 5). Each lever ran out once it had been used. After that, the next gain came from a different part of the path.
- **Ghost or history signals work only with a probationary queue to bypass.** The S3-FIFO split (small FIFO + main SIEVE + ghost) gave the first hit-rate gain in several cycles and did not cost latency.

## Recurring failures

- **Stacking scan-protection mechanisms.** BIP inside SLRU probation (cycle 2) shrank the window in which new keys can earn a second touch, and hit-rate dropped. These mechanisms overlap. They do not add up.
- **Tuning how the hand treats queued nodes.** Deeper per-slot credit (GCLOCK cycle 2, the 2-level SIEVE counter cycle 3) and sweep caps (cycle 3) both cost hit-rate. Deeper credit also made the put tail worse. SIEVE's single visited bit looks like a local optimum.
- **Blaming long hand sweeps for the put tail.** Capping the sweep did not lower put p99 (cycle 3). The tail came mostly from map work on the miss path. Hashing changes and structural eviction changes have moved it. Hand tweaks have not.
- **Adding ghosts to a single queue.** In cycle 4, ghosts on a single queue extended the stay of keys whose reuse distance was too long. Those keys pushed out better residents and added hashing on every miss.
- **Hand-rolled key-less hash tables.** In cycle 5, a hand-rolled table could not reject a bucket without dereferencing the arena. Backward-shift deletion made clusters expensive. hashbrown's tag filtering beats it.
- **Node-shape work aimed at speed** after relinking was removed. Since cycle 3 it has only saved memory.

## Open questions

- **Generalization:** every result so far comes from one workload, workload-a, at size 2048. Testing a second size or trace has been recommended for five cycles and has never been done. This matters most now that a policy change (S3-FIFO) has won.
- **S3-FIFO knobs:** how much hit-rate is left in the small-queue ratio and ghost size, and in whether main-queue evictions should feed the ghost? The modest gain suggests workload-a has limited one-hit traffic.
- **A single-probe miss path:** can hashbrown's entry or raw-table API give one hash per miss while keeping tag filtering?
- **Hit-path locality:** can the visited bit sit next to the data a hit already reads, so a hit touches one memory location instead of two?
- **Do the remaining gains stack?** Specifically Fx hashing, the side array, swapping the key on eviction, and S3-FIFO. Fx and the side array did not stack on get latency in cycle 5.
- **Metadata accounting:** heap memory for ghosts and side arrays, and index density, are invisible to the per-node metric. Judgments about metadata need bytes counted per entry.

## Per-policy notes

- **LRU (baseline):** strict recency order is a liability on workload-a. It has been dropped as a direction since cycle 1.
- **SLRU:** second-touch promotion and `u32` links helped in cycle 1. Head insertion into probation hurt in cycle 2. If SLRU is revisited, change the probation size or admission, not the insertion position.
- **CLOCK / GCLOCK:** a hit path with no relinking and a smaller node won in cycle 1. GCLOCK's 2-bit credit bought hit-rate at a visible put-p99 cost in cycle 2. Further CLOCK tweaks have been dropped.
- **BIP:** works alone as a loop-resistant insertion policy (cycle 1). It works against admission windows when layered inside SLRU.
- **SIEVE:** the best single-queue policy, with one bit and quick demotion of new keys. The singly-linked version with a side-array layout and Fx hashing is its leanest form. Changes to counters, sweep caps and single-queue ghosts all regressed.
- **S3-FIFO (small FIFO + main SIEVE + fingerprint ghost):** the current parent. It is the first policy change to improve both hit-rate and the put tail. Its ghost heap is not yet counted in metadata. Its knobs are untuned, and it has not been verified beyond workload-a at size 2048.
