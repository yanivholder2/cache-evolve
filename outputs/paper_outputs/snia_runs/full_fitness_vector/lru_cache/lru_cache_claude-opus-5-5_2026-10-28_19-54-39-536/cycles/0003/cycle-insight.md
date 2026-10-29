# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle-0002 v003 (flag-packed SLRU on `u32` nodes). Two added per-cache history state: an A1out ghost (v001) and a miss-only TinyLFU admission gate (v003). The third specialised the hit-path relinks (v002). The Winners are v002 and v003, but neither clearly beat the parent. v002 kept policy and layout exactly the same and moved per-op-time only slightly, with mixed direction. v003 lost hit rate overall and added put cost.

## Patterns

- Per-cache side structures are not free for get, even when get never touches them. Both the ghost set and the sketch table made get p50 worse, most likely through CPU-cache pressure on the shared map and arena. The rule "keep it off the hit path, keep it out of the node" does not protect the summary latency.
- History-based admission failed in both forms, and in opposite directions. The ghost was too eager: returning keys went straight to protected and demoted hot entries, which hurt web_0. The sketch was too strict: counting only misses and rejecting ties turned it into a "must miss twice" filter that starved short-reuse keys, again on web_0. hm_0 gained a little under both. web_0 at the large size is still unsolved and now looks sensitive to how aggressive admission is.
- Relink micro-optimisation is used up. v002 confirms what cycle 0001 suggested about get: the list work is not the bottleneck. Removing branches and stores was lost in noise. The early-exit branch, and the larger inlined hot code, may have cost as much as they saved.
- Every variant that added a hashed structure (ghost `HashSet`, SipHash sketch) increased put cost by roughly one hash operation per probe. The put path is bound by hashing and memory, not by list logic.

## Open questions

- Would a much smaller or fingerprint-only ghost with gentler admission (probation MRU, or protected without forced demotion) keep the hm_0 gain without hurting web_0?
- Would W-TinyLFU (a sketch that counts hits too, a small admission window, a cheap hash) do better than the bare gate?
- Is the get p50 regression from side structures real cache pressure or benchmark interleaving noise? No variant isolated this.
- Map/arena locality and avoiding the key clone on insert are still untested as per-op-time levers.

## Continuation

Keep cycle-0002 v003 SLRU as the base. v002's specialised relinks are a safe but optional refinement, not a speed win. Drop further relink and branch trimming. Drop full-key ghosts and miss-only, tie-rejecting admission gates. If you pursue hit rate, any admission or history state must be tiny and cache-friendly, and must admit gently. If you pursue latency, target map/arena locality or insert-path allocation and cloning, not list maintenance.
