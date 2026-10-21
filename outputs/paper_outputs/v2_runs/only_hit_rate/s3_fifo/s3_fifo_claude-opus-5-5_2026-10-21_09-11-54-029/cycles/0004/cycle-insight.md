# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle 0003's v003 (5% split, cap 7) and each changed one thing. v001 ported that policy onto the indexed slab/index/lazy-ghost structure without changing behaviour. v002 raised the frequency cap from 7 to 15. v003 sent ghost hits back through the small queue with one banked credit. v002 is the nominal Winner, but its hit-rate uptick is within noise. v001 matched its parent exactly, as intended, and v003 regressed clearly.

## Patterns

- The frequency cap stops paying off at 7. Main-queue evictions mostly take freq-0 ghost re-entries, which drain long before a hot key runs out of credit, so extra credits rarely change which key gets evicted. Main retention is now flat on the cap axis. The Winner's only certain effect is longer eviction sweeps, and that cost is still unmeasured.
- The re-admission question is now settled from both sides. Tail insertion (cycle 0003) and small-queue probation (this cycle) both lost, and extra credit (cycle 0002) also lost. A bare ghost hit is enough evidence of reuse. Front-of-main at freq 0 is the right re-admission point. v003 showed why the probation idea fails: the small queue's window is much shorter than the ghost's reuse horizon, so it rejects the keys the ghost correctly found. Routing re-entries through small also crowds out cold keys that are trying to graduate.
- The structural port requested for three cycles now exists on the current policy, verified step by step against the linear reference. Policy and representation are separate. But per-op-time and metadata-size were again not reported, so nobody can yet say whether the port helps. Four cycles have now passed without those measurements.

## Open questions

- How much per-op-time does the indexed port save, and how much metadata-size does it add (duplicated resident and ghost keys, a stale ghost deque, occasional compaction spikes)?
- Does the cap of 15 cost per-op-time on the linear structure? If it does, the Winner is a net loss.
- With partition, cap and re-admission all flat or settled, is there any hit-rate headroom left on workload-a? Credit aging or decay is the main untested retention lever.

## Continuation

Treat the cap of 15 as a hit-rate tie. Unless per-op-time shows the longer sweeps cost nothing, use cap 7 on v001's indexed structure as the effective baseline. Drop all ghost re-admission variants (credit, tail, small-queue probation) and stop raising the cap. Make the evaluator report per-op-time and metadata-size the first priority, because the remaining open questions depend on them. After that, the only hit-rate lever worth probing is how main-queue credit is aged or spent (e.g., decay), not how much of it there is. Also check whether the gains hold on other traces or cache sizes.
