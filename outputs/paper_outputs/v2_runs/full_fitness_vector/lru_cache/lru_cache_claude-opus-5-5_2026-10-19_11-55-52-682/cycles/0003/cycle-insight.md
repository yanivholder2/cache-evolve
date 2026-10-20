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
