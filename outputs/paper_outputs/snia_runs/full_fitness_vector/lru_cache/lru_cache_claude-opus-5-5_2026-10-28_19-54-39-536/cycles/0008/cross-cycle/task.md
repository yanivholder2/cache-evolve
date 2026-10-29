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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Two variants branched from cycle-0005 v001 (the size-gated `prev`-bit reprieve). v001 cut the ghost's hashing cost (Fx tags, vacant-path only, single-OR `set_protected`). v003 turned the reprieve back on at 131K with a much smaller ghost, sized at capacity/8. v002 branched from cycle-0004 v002 and replaced hashbrown with a custom tombstone-free index table. All three are Winners, but each won only partly. v001 got most of the median put cost back with no hit-rate loss. v002 fixed the get tail but lost get p50. v003 was roughly neutral on hit rate at 131K.

## Patterns

- Two get-p50 explanations have now been ruled out. v001 removed the flag arithmetic from `on_hit` and get p50 still regressed. v002 touched fewer cache lines per lookup and still lost to hashbrown's SIMD group probing. So get p50 does not depend on flag bit-ops, list work or lookup cache lines. The leading suspects left are code layout and inlining of the shared helpers, and the ghost table's cache footprint.
- Put cost tracks hash count, now for the fourth cycle. A cheap fingerprint hash recovered the put median, and a 16-bit tag does not need a strong hash. v002's backward shift, which runs a full SipHash for each entry it moves, pushed the put tail up the same way.
- Removing the victim before inserting the new key fixes the get tail. v002 is the first evidence that the cycle-4 get-p99 rise was real and came from tombstone churn, not noise.
- Noise now hides the effects this cycle measured. v003's timings swung sharply on code that was byte-identical to v001. Its 16K hit rate moved on unchanged code, most likely because `RandomState` is reseeded on every run. Small hit-rate deltas cannot be trusted while the ghost hasher is unseeded or randomly seeded.
- An undersized ghost works as a reuse-distance filter, but at capacity/8 it is too short for web_0 at 131K. It forgets most of the useful re-references before they come back.

## Open questions

- Does a wider large-size ghost (capacity/4 or capacity/2) recover cycle 4's web_0 gain at 131K while still keeping out hm_0's long-loop keys?
- Is the remaining get-p50 regression caused by inlining and layout of `on_hit`/`unlink`/`push_back` after the put restructure? Forcing or blocking inlining of those helpers would test this.
- Does remove-before-insert on the stock HashMap parent get v002's tail fix without losing hashbrown's probing speed?

## Continuation

Use cycle-0006 v001 (Fx ghost tags, vacant-only ghost work) as the base. Fix the ghost hasher's seed first, so hit-rate deltas can be measured, and time parent and child in repeated paired runs. Then try remove-before-insert eviction on hashbrown, and a large-size ghost wider than capacity/8. Drop the custom scalar index table, and drop further bit-op trimming in `on_hit`. Look into the get-p50 regression through helper inlining and code layout.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two variants targeted latency on the cycle-0004 v002 base without changing policy. v001 split eviction and promotion into cold functions and forced the hot helpers inline. v003 removed the victim before inserting the new key on stock hashbrown. v002 turned the cycle-0006 v001 reprieve on at 131K with a capacity/2 ghost. The Winners are v001 and v002. v002 is the clean gain: web_0 at 131K improved more than hm_0 dipped, and 16K was unchanged. v001 won only on the get tail, and its get p50 got worse.

## Patterns

- Two unrelated changes both "fixed" cycle 4's get-p99 rise. One was a pure layout split (v001), the other a change in eviction order (v003). Both also made get p50 worse. When different causes produce the same effect, noise or broad layout sensitivity is a more likely explanation than any one mechanism. Neither claimed cause has been established.
- Get p50 still resists every lever. Forced inlining made it worse, and v003 changed no get-path instruction and it still rose. Under a p50 summary, trading the median for the tail has now lost in two variants.
- Put cost tracks hash count, now for the fifth cycle. v003's extra SipHash on every evicting put undid the single-lookup saving.
- The ghost window works as a reuse-distance filter. Cycle 6 showed capacity/8 is too short for web_0 at 131K. capacity/2 catches most of web_0's useful re-references and lets in only a few of hm_0's loop keys. The best width probably lies between the two. With the zero-seeded Fx ghost, hit-rate deltas can now be trusted.
- `metadata-size` does not count the heap ghost table. At 131K the ghost is a large hidden memory cost.
- Two of the three variant insights were written without access to the overlay source. Their explanations assume the hypothesis was implemented as written.

## Open questions

- Is capacity/4 (or stricter reprieves at large sizes) a better point on the web_0-gain/hm_0-leak trade-off than capacity/2?
- Does v001's `#[cold]` split without the `#[inline(always)]` hints keep the tail fix without the get-p50 loss?
- Is any of this cycle's get-p50 or get-p99 movement real? None of it has been confirmed by repeated paired runs.

## Continuation

Use cycle-0007 v002 (capacity/2 ghost at 131K) as the new base. Tune the large-size ghost width toward capacity/4 or tighten large-size reprieves. Consider counting the ghost table in metadata. Drop forced `#[inline(always)]` and drop remove-before-insert using a second hash. Retry an eviction-order change only through a hash-preserving path (`HashTable`/raw entry). Stop chasing get p99 or p50 until paired repeated runs show the effect is larger than noise.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Two variants branched from cycle-0007 v002 (capacity/2 ghost at 131K) and made the ghost smaller. v001 narrowed the large-size ghost to capacity/4. v002 cut the ghost tags from 16 bits to 8. v003 branched from the ghost-free cycle-0004 v002 and widened probation to about 1/3, at 131K only. All three are Winners. v001 is the hit-rate gain: a small summary rise, plus timing gains that are probably partly noise. v002 is the clearest latency result so far: get p50 improved with bit-identical hit-rate. v003 is a weak Winner: neutral to slightly negative on the summary, and most useful as a noise-floor and negative control.

## Patterns

- The ghost table's byte footprint slows get, even though get never reads the table. Both variants that shrank the ghost (v001 fewer slots, v002 narrower tags) had lower per-op times on unchanged get code. v002 changed only the tag width, so it is the first variant to isolate a cause of the long-running get-p50 regression. The footprint suspect from cycles 6–7 is now the leading explanation, not a guess.
- At 131K, hm_0 and web_0 pull in opposite directions on any probation-time lever. A narrower ghost (v001) and a static wider probation (v003) both traded hits between the two traces. Only the ghost-targeted version came out net positive. This repeats cycle 5's result at 16K: selective reprieves beat a uniformly wider window at both sizes.
- Ghost width tuning is nearly used up. capacity/4 is slightly better than capacity/2, and the moves on both traces are now marginal.
- The noise floor is now visible. v003 left the hot paths identical, yet its per-op times moved noticeably. Any per-op delta of similar size in v001 or v002 should not count as evidence on its own. v002's get-p99 rise has no code cause and fits this noise.
- v002's hit-rate was bit-identical even though 8-bit tags collide far more often. This is plausible, because spurious reprieves are cheap. It could also mean the harness never exercises those collisions. This has not been checked.

## Open questions

- Do v001 and v002 stack? A capacity/4 ghost with 8-bit tags would be a quarter of the parent's ghost bytes at 131K. Does get p50 keep falling?
- Can the ghost get smaller still, for example with 4-bit tags, a bitset or a Bloom-style ghost, before the hit-rate cost becomes visible?
- Are spurious reprieves actually happening in the hit-rate harness? Count them.
- Is v002's get-p50 gain larger than v003's noise swing when measured in repeated paired runs?

## Continuation

Make the base cycle-0008 v001 combined with v002's 8-bit tags (capacity/4 ghost, u8 tags). Keep shrinking ghost bytes as the latency lever, and add a spurious-reprieve counter to confirm hit-rate neutrality. Drop static probation/protected split retuning at every size, and drop further ghost-width tuning. Any new hit-rate lever at 131K should change how reprieves are granted, not the window size. Report get-p50 claims against v003's noise floor and confirm them with paired runs.


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
