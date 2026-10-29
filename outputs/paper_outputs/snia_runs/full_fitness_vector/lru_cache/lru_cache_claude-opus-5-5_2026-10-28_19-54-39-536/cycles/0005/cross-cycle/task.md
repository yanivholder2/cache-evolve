# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from baseline, each aimed at a different axis: SLRU for hit rate (v001), CLOCK lazy promotion for get latency (v002), and `u32` link compaction for metadata size (v003). Both Winners (v002, v003) shrank node metadata. v001 improved hit rate, but paid for it with larger nodes and slower operations.

## Patterns

- Shrinking metadata was the most reliable win. Two different routes got there: dropping the links entirely (v002) and narrowing them (v003). The node layout has no free padding, so any added field costs a full word unless it reuses space the node already has. v001's prediction that padding would absorb its new byte was wrong.
- get latency hardly moves whatever the list does. Removing relinking (v002) and compacting nodes (v003) both left get p50 roughly flat. This points to the `HashMap` probe, not list maintenance, as the dominant cost of a hit.
- Frequency or scan filtering gives modest hit-rate gains on the MSR block traces. The gains depend on cache size. SLRU helped mostly at the large size, while CLOCK's single bit helped hm_0 at both sizes and web_0 at the small size, and lost slightly on web_0 at the large size. Neither approach dominated.
- Policy changes moved several axes at once, with mixed results. Representation-only changes (v003) moved one axis cleanly and left hit rate exactly unchanged.

## Open questions

- Can SLRU's frequency filter run on a compact node (a segment flag in a link high bit or sentinel, on top of `u32` indices) without the metadata and latency penalty?
- Would an adaptive or larger probation share fix SLRU's weakness at the small cache size?
- Can CLOCK's put tail latency be bounded, for example with a capped sweep or amortised hand advance, without losing its median gain?
- Would a faster hasher or a different map layout finally move get p50?

## Continuation

Treat v003's `u32`-index layout as the default base for later variants. It is policy-neutral and costs nothing. Combine it with scan resistance: try SLRU with an in-node flag and/or a tuned probation share, or CLOCK with a bounded sweep. To lower get latency, target the hash map. Drop further list or node-compaction ideas as a speed lever.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Every variant was a follow-up to a cycle-0001 open question. v001 swapped the hasher on CLOCK, v002 added a GCLOCK counter in CLOCK's padding byte, and v003 ported SLRU onto the `u32` node with an in-link flag. All three are Winners, but only v003 got a clear gain without a matching cost. It kept SLRU's full hit-rate benefit on a compact node, and its remaining price is confined to put.

## Patterns

- Folding policy state into existing bits now works. A segment flag in a link's high bit (v003) and a counter in the padding byte (v002) both kept the node at its compact size. So the metadata objection to frequency- or scan-aware policies is settled. What's left to fix is latency.
- Hit-rate gains came with put-path costs in both policy variants. In SLRU the cost is the promote/demote double relink plus branching per segment. In GCLOCK it is sweep length multiplied by decrement-by-one aging. Neither added meaningful cost to get p50, so the work is in the put/eviction path, not the hit path.
- SLRU beat GCLOCK on hit rate: it improved every trace at both sizes, while GCLOCK's gain was mostly hm_0 at the large size. Neither variant fixed web_0 at the large size, so plain frequency counting is not what that trace needs.
- The hash-map lever failed again, and this time get got worse. Making the hash cheaper doesn't help. The probe appears to be bound by memory and sensitive to hash distribution on structured block keys. This is inferred, since probe lengths aren't measured.

## Open questions

- Can SLRU's put cost be cut? For example, avoid the second relink on promotion, give puts to existing keys a cheaper path, or remove the per-segment head/tail branching.
- Was GCLOCK's get slowdown real, or an artifact of the long put sweeps interleaved with gets in the benchmark?
- Would a denser or smaller table, such as narrower values or a better-mixing finaliser, move get latency where a cheaper hash did not?
- What does web_0 at the large size need, given that neither recency segmentation nor small counters fixed it? Possibly ghost/history entries or an adaptive split.

## Continuation

Make v003's flag-packed SLRU on `u32` nodes the default base, and next try to cut its put-path cost: cheaper promotion/demotion and a fast path for updating existing keys. Drop cheaper-hash ideas. Drop GCLOCK with decrement-by-one aging unless aging is moved off the eviction path, for example by bulk halving. If you go after get latency, try table density or hash quality, not hash speed.


### Cycle 0003

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle-0002 v003 (flag-packed SLRU on `u32` nodes). v001 tried an adaptive protected/probation split, v002 a single-`Entry` put path with no key clone, and v003 a gentle fingerprint-ghost second chance inside probation. The Winners are v002 and v003. v002 is the only clean gain: put got faster with policy and layout unchanged. v003 is the first history mechanism that helped web_0 instead of hurting it, though the net hit-rate gain is small and get latency regressed.

## Patterns

- Cutting hash operations on put works. v002 bears out the cycle-0003 finding that put cost tracks hash count. It is the first latency lever across four cycles that moved its target without hurting hit rate or metadata.
- Gentle admission solves web_0's sensitivity. A one-shot reprieve that stays in probation lifted web_0 at both sizes, which reverses the damage cycle 3's ghost and gate did. Its benefit is size-dependent, though: hm_0 gained at the small size and lost slightly at the large size.
- Small per-cache hit-rate effects are hidden by noise. v001's one-epoch hill-climber kept chasing workload swings and lost on hm_0. Its only win was web_0 at the small size, which again suggests probation is undersized there.
- Any extra work on the hit path is very costly for get. v001's per-op tick and v003's masking of `next` both raised get p50 sharply. Header growth past one cache line and heap side arrays probably add to this. Both policy variants also grew the `LruCache` header. Repeated across cycles, the rule is that the `on_hit` link path and the header's hot cache line must stay untouched.
- v002's get p99 rose although its get code was identical. This may be noise, or the insert-before-remove order may be changing the hash table's internal state. It is unresolved.

## Open questions

- Is v002's get-p99 rise real? If so, would removing before inserting (so the new key reuses the freed slot) keep the put gain without the tail?
- Can v003's second-chance bit live somewhere that is free on the hit path, such as a padding byte or the key/value side rather than `next`, so get p50 recovers?
- Should the reprieve be gated by cache size, applying only when probation is small relative to the working set, so it stops costing hm_0 at the large size?
- Would a static per-size retune of the probation share capture v001's web_0 small-size gain with no runtime cost?

## Continuation

Use v002's single-`Entry`, no-clone put as the new default base. Re-check its get p99, and if it is real, try remove-before-insert. If you pursue hit rate, port v003's probation-only reprieve onto that base, with the flag stored off the `next` link and the header kept within one cache line, and consider scaling it by cache size. Drop online hill-climbing of the SLRU split. Try a static per-size probation share instead.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle-0004 v002 (single-`Entry`, no-clone SLRU). v001 ported the fingerprint reprieve with its flag in `prev` and the ghost enabled only for small caches. v002 replaced protected-hit relinks with a CLOCK reference bit. v003 used a static 1/3 probation share at small sizes. v001 is the only Winner. It lifted both traces at 16K and matched the parent at 131K, but gave back v002's put gain by doubling hash work, and get p50 still regressed.

## Patterns

- Gating by size works for hit-rate mechanisms. Both v001 and v003 left 131K untouched by design, so every hit-rate change came from 16K alone. v001's gate removed the large-size hm_0 loss from cycle 4.
- At 16K the reprieve beats resizing. A static wider probation helped hm_0 but hurt web_0, so no fixed share suits both traces. The reprieve gives extra probation time only to keys that come back, and it helped both. Selective second chances do better than a uniformly bigger window.
- Put cost still tracks hash count. v001's separate `hash_one` for ghost tags, on top of the map's own hashing, erased the single-probe gain. This is the third cycle confirming the rule.
- Hit-path changes still hurt get p50, even tiny ones. An extra AND in `set_protected` (v001) and CLOCK moving list work into promotion sweeps (v002) both regressed it. Removing relinks from protected hits did not lower the median. That is more evidence that get p50 is not driven by list work.
- The timing benchmark is too noisy to rank small per-op changes. v003 changed nothing on the hot path, yet its per-op times swung widely in opposite directions. Get p99 swings in every variant now look like noise, including cycle 4's p99 rise.

## Open questions

- Can v001's ghost reuse the hash the map already computes (raw-entry or `hashbrown` API), and run only on the vacant path, to restore v002's put cost?
- If `set_protected(true)` is kept to a single OR and `SC_BIT` is cleared at reprieve or eviction time, does get p50 recover?
- How much timing movement is just run-to-run variance? Repeated paired runs of parent and child are needed before trusting per-op deltas under a few percent.

## Continuation

Make v001 (size-gated `prev`-bit reprieve) the base, and fix its costs next. Derive the ghost tag from the map's own hash, skip ghost work on occupied-key puts, and make `set_protected` a single OR. Drop protected-segment CLOCK and static probation-share retuning. Judge latency claims only from repeated paired runs.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
