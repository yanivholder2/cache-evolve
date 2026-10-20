## Recurring wins

- Leaving strict LRU order pays off. Every policy that stopped relinking on each hit or protected reused keys from scans gained hit-rate on workload-a. That covers SLRU, CLOCK, BIP, GCLOCK and SIEVE.
- A hit path with no relinking (hand-based policies) keeps get latency low in every cycle where it was tried.
- Node compaction is a reliable metadata lever: `u32` index links, fewer links (CLOCK, singly-linked SIEVE), and policy bits stored in padding bytes. Once the hit path stops relinking, compaction no longer speeds things up; it only saves metadata.
- Rewriting the representation while keeping the eviction order identical is the safest way to improve. It has never cost hit-rate.
- Simple beats layered: SIEVE (one queue, one bit, quick demotion of new keys, lazy promotion of reused keys) matched or beat the segmented policies.

## Recurring failures

- Adding more in-queue reuse signal has not helped. Deeper counters (GCLOCK, the 2-level SIEVE counter) cost put-tail latency and, on SIEVE, hit-rate as well.
- Scan defenses don't add up. Putting BIP head insertion inside SLRU probation shrank the window new keys have to earn a second touch, and hit-rate dropped.
- Cutting the sweep short (capping the hand's steps) evicts runs of visited nodes that carry real reuse. It cost hit-rate and did not fix the put tail.
- "Labeled Winner" is a weak signal. Several winners had flat or negative results on the main dimensions, so judge each one by what actually moved.

## Open questions

- Does the ranking generalize? After three cycles every result still comes from workload-a at size 2048. This is the top unaddressed risk.
- Can history of evicted keys (ghost entries, S3-FIFO small/main/ghost, ARC/LIRS-style tracking of keys no longer in the cache) add hit-rate where in-queue counters could not?
- What actually sets put p99? Sweep length has been ruled out. Suspects are the per-miss HashMap remove/insert, the key clone and cold-node touches.
- Are SLRU's probation sizing or an admission filter still worth testing, given that changing where new keys are inserted failed?

## Per-policy notes

- LRU (baseline): beaten on every axis by every alternative tried. Reordering tweaks have been dropped.
- SLRU: second-touch promotion gives solid hit-rate, and `u32` links shrink the node. BIP inside probation hurt. Any future change should target the segment split or admission, not where new keys are inserted.
- CLOCK / GCLOCK: the reference-bit ring is cheap and gained hit-rate unexpectedly. GCLOCK's deeper credit buys hit-rate at a visibly worse put tail.
- BIP: works as a standalone insertion policy with an early speed gain whose reliability is uncertain. It conflicts with probation-based schemes.
- SIEVE: the current best lineage and parent. Singly-linked SIEVE (the hand carries its predecessor) is the most compact version with unchanged behavior. Counter-depth and sweep-cap variants are exhausted. Next steps are ghost/history signals and work on the HashMap path on misses.
