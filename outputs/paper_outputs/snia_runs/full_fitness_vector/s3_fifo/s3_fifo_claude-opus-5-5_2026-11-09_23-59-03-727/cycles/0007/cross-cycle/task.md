# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-axis changes to how S3-FIFO splits capacity and history: a fixed 10% small queue (v001), ghosting main-queue victims (v002), and an ARC-style adaptive small/main split (v003). None of them clearly beat baseline on hit rate. v001 and v002 came out roughly neutral, trading gains on some trace/size points for losses on others, and v003 regressed.

## Patterns

- Gains and losses depend on the trace and the cache size. hm_0 tends to reward more main-queue residency or main-victim history. web_0 at the large size keeps losing whenever the probation window or the ghost history meant for small-queue demotees gets squeezed. Each change helped one corner of the trace×size grid and hurt the opposite corner.
- Ghost capacity is a shared, zero-sum resource. v002 and v003 both kept the ghost bound fixed while changing what competes for it or how big main is, and both showed the mismatch hurting. The ghost bound looks like a hidden coupled variable that cycle 1 never isolated.
- Every variant showed a large per-op-time improvement that its diff can't explain, including v003, which only adds work. This is a benchmark-environment shift, not a real effect. Per-op-time comparisons from this cycle are unreliable.
- Metadata per entry stayed flat across the cohort. Only v003 added a small fixed per-cache cost.

## Open questions

- Is the per-op-time drift caused by the baseline being measured in a different environment? Until a same-machine baseline is re-run, no variant should get timing credit.
- Would a ghost sized or split independently of `main_size`, for example with separate regions for small demotees and main victims, let v002's hm_0 gain survive without the web_0-large regression?
- Can adaptive sizing work if the two signals are balanced (normalized by queue size, or shrinking once per evicted main entry rather than per reinsertion), the ghost tracks the live main size, and the step is smaller at large capacity?

## Continuation

Drop fixed-ratio sweeps. A fixed ratio only moves the trade-off between hm_0 and web_0-large around without improving the summary. Next, try making ghost capacity its own variable, either decoupled from or split by source, and possibly a rebalanced adaptive split with a ghost that tracks main's live size. Re-baseline per-op-time in the same environment before trusting any timing result.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants were policy tweaks on top of cycle-1 parents: v001 split the ghost into a small-demotee region and a main-victim region, and v002 lowered the graduation threshold to one hit and reset freq on promotion. A third variant, v003, kept the policy and replaced linear scans with a slab plus hash indexes. The two policy tweaks again moved hit-rate gains between hm_0 and web_0 without a net win. v003 matched its parent's hit rates exactly, and it is the first per-op-time gain that the diff itself explains.

## Patterns

- hm_0 keeps rewarding anything that gives reused keys more main residency or history: restoring the demotee ghost, keeping main-victim ghosts, or promoting after one hit. web_0 regressed under both policy variants, so cycle 1's trade-off between the two traces is still in place.
- Both policy hypotheses got web_0's mechanism wrong. Restoring demotee ghost history in full did not rescue web_0 at the large size, which rules out ghost-slot competition as the cause there. The one-hit threshold only affects keys that were already reused, while web_0's loss involves freq-0 keys. The likelier culprits are freq-0 keys re-admitted from the main-victim ghost straight into main, plus the freq reset on promotion. v002 bundled that reset with the threshold change, so the two effects can't be separated.
- Policy and implementation now separate cleanly. v003 shows that structural speedups can be made without changing hit rate, so timing credit can go to real complexity changes. Unexplained timing drift in v001 and v002 is still environmental noise.
- The metadata-size dimension (per-entry footprint) stays flat even when heap use per key grows. Both v001 (ghost key storage) and v003 (hash maps, stale ghost tuples) added memory the metric doesn't see.

## Open questions

- Does ghosting only main victims that once had a nonzero frequency keep hm_0's gain without hurting web_0?
- On web_0, is the freq reset on promotion or the looser threshold the cause of the regression? Test each alone.
- Should metadata-size be supplemented with heap bytes per resident key, now that hashed structures are on the table?

## Continuation

Use v003's slab-plus-hash structure as the base for future policy variants: per-op-time is now solved structurally and comparisons will be faster. Stop resizing ghost regions. Next, test whether main victims should be ghosted at all, for example only victims that had a nonzero frequency or re-admitting ghost hits with some credit. Test the freq-reset-on-promotion change on its own, separate from the threshold. Make one change per variant so web_0's mechanism can finally be pinned down.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants were single-change children of cycle-2 v003 (slab + hash). v001 ghosts only main victims that were hit while resident. v002 resets freq on promotion from small to main, alone. v003 swaps SipHash for an Fx-style hasher. All three are Winners, and this is the first cycle where neither trace regressed: the two policy variants each gained slightly on hm_0 and held or gained on web_0, and the hasher swap gave a timing gain that its diff explains, with hit rates bit-identical.

## Patterns

- Isolating one change per variant worked. The cycle-1/2 trade-off between hm_0 and web_0 went away once changes stopped being bundled. Both policy variants filter out "credit without reuse in main", either as ghost slots or as carried-over freq, and both help hm_0 without costing web_0.
- web_0's mechanism is now mostly pinned down. The freq reset is harmless, so cycle-2's web_0 loss came from the looser `freq >= 1` graduation threshold. Unreused ghost re-admissions are at most a minor factor: web_0 at the small size didn't move under v001, and its large-size gain was only about the size of hm_0's.
- Filters that free up ghost budget lengthen the useful history for free. This fits cycle 1's finding that the ghost bound is a hidden coupled variable.
- Timing noise is still a problem. Both policy variants showed per-op-time regressions their diffs can't explain. Only structural changes (v003 now, and cycle-2 v003) earn timing credit.
- Per-entry metadata stayed flat throughout. v003 also shrank the fixed per-cache struct.

## Open questions

- Do the v001 gate and the v002 reset stack additively? They touch different mechanisms, so combining them is the natural next test.
- What explains the rest of web_0's large-size gap? The remaining candidates are small-queue sizing and the ghost bound itself.
- Do the v002 `put`-tail shifts come from more frequent main evictions or ghost compaction, or from noise? Only a fixed-environment re-run can tell.

## Continuation

Merge the three Winners into one parent: Fx hashing + reset-on-promotion + reuse-gated main-victim ghosting, with the `freq > 1` threshold kept. Confirm the policy gains add up before testing anything new. After that, probe what's left of web_0's large-size loss, one variable per variant: ghost bound relative to main size, or small-queue share. For implementation, cut hash probes per miss with the entry API. Keep refusing timing credit for policy-only diffs until the benchmark environment is pinned.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort had three single-purpose tests: v001 merged cycle-3's two filters (reuse-gated main-victim ghosting plus freq/reused reset on graduation), v002 raised the ghost bound from `main_size` to the full capacity, and v003 merged the resident and ghost indexes into one map. Only v001 was a real win. It improved hit rate at every trace/size point, with entry size unchanged and no latency cost. v002 regressed web_0 and large hm_0, and v003 kept hit rates bit-identical but got slower. Both count as Winners only by rank. Neither should be built on.

## Patterns

- The filters add up. Gating ghosts on reuse and resetting credit at graduation stack cleanly once both judge a key only by its reuse inside main. hm_0 gains more than web_0, which matches every cycle so far: hm_0 responds to how main retains keys.
- More ghost history is harmful, not neutral. A longer window lets stale reuse back into main, where it pushes out live residents. Cycles 3 and 4 together point one way: ghost budget should be spent more selectively, not made larger. This rules out the ghost bound as the cause of web_0's remaining large-size gap.
- Probe count is not the bottleneck for timing. Halving probes lost to higher load factor and wider buckets, which hurt the tails most. Table occupancy and bucket width matter more than the number of lookups. Structural timing work now needs to budget for memory layout as well as the algorithm.
- Fixed-struct size keeps shifting (v002 up, v003 down) while per-entry metadata stays flat across the cohort. The summary metric still misses live ghost memory.

## Open questions

- Is the small-queue share (`capacity / 4`) what's left behind web_0's large-size loss? It is the last remaining candidate.
- Now that the merge works, do the graduation threshold or `MAX_FREQ` give more gain on hm_0 without costing web_0?
- Would a unified index pay off with generous headroom or a compact slot encoding (a `u32` slab index plus a ghost flag bit), or is the two-table layout already close to optimal?

## Continuation

Use v001 (Fx slab + both filters) as the only parent. Keep the ghost bound at `main_size` or lower, and drop the idea of a longer ghost. For web_0, test the small-queue share one variable at a time. For hm_0, try main-retention knobs (`MAX_FREQ`, graduation threshold) as separate variants. If you retry index merging, pair it with explicit load-factor headroom or a compact slot. Otherwise, drop structural probe-count work. As before, give no timing credit to policy-only diffs.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants were single-change children of cycle-4 v001 (Fx slab + both filters). v001 shrank the small queue from 25% to 10%, v002 raised `MAX_FREQ` from 3 to 7, and v003 pre-sized both hash indexes to twice their occupancy. Only v002 gained, and only slightly in summary, mostly on hm_0 at the small size. v001 regressed web_0 at both sizes and lost on the summary. v003 kept hit rates bit-identical but made tails worse. All three count as Winners by rank only. v002 is the only one worth carrying forward.

## Patterns

- Every gain this cycle sits at the small cache size, and mostly on hm_0. A bigger main (v001) and a deeper freq cap (v002) both helped hm_0 at 16384 and did nothing or hurt at 131072. Main retention is close to saturated at the large size. Whatever is left on hm_0 is a small-cache problem.
- web_0's large-size gap is a probation-window problem. Shrinking the small queue widened the gap, so web_0 needs new keys to stay in probation long enough to get the second hit that graduation requires. All structural candidates (ghost bound, small-queue share) have now been tested. What remains is to make probation longer or graduation easier, not to resize main.
- The layout lesson from cycle 4 is about footprint, not load factor. More headroom hurt tails just as the denser merged index did. hashbrown's rounding already leaves the resident index about half empty. Memory footprint and locality drive the `put` tail, so the only layout direction left is shrinking tables, especially the ghost index.
- `put` p99 moved slightly up in every variant: longer ghost compactions (v001), longer second-chance sweeps (v002), sparser tables (v003). Only v003's regression is clearly caused by its diff. The other two shifts are within noise.
- Per-entry metadata stayed flat again, while v003 doubled index heap without the metric showing it.

## Open questions

- Does a larger small queue (for example 1/3), or a softer graduation threshold applied only inside probation, close web_0's large-size gap without bringing back cycle-2's `freq >= 1` regression?
- Would a size-aware `MAX_FREQ`, or a split between small and main that depends on cache size, keep v002's small-cache hm_0 gain without its large-size flatness?
- Can the ghost index be made denser (for example by storing fingerprints or compact keys) to cut `put` tail cost?

## Continuation

Use v002 (MAX_FREQ 7 on the two-filter parent) as the new parent. Raising the cap further is unlikely to pay off. Drop small-queue shrinking and index over-allocation. For web_0, test one variable at a time: a longer probation window (small share above 25%) or easier graduation inside probation. For implementation, only shrink tables, with the ghost index first. As before, give no timing credit to policy-only diffs.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002 (MAX_FREQ 7 on the two-filter parent). Each tested one way to fix the cycle's remaining gaps: v001 grew the small queue to 1/3, v002 gave freq-1 small keys one reinsertion into small, and v003 re-admitted main-victim ghost hits with `freq = 1`. None clearly beat the parent. v002 and v003 each gained a sliver in one place and lost a sliver in another, and v001 lost slightly everywhere. All three are Winners by rank only, and none should replace the parent.

## Patterns

- web_0 at the large size slipped in every variant, the cell two of the three hypotheses targeted. Cycle 5's reading that this gap is about probation length is now falsified from both sides. Shrinking small (cycle 5) and growing it (v001) both regressed, so the 1/4 share sits at a local optimum. Extending probation only for freq-1 keys (v002) also failed to help.
- The ghost queue already acts as the second chance for demoted keys. v002's in-small reinsertion mostly duplicated the ghost→main path and pushed out newcomers. v003's extra credit on the ghost→main path helped hm_0 but let stale returners displace live residents on web_0. Whatever web_0 lacks sits in how the ghost hands keys to main, not in what happens inside small.
- hm_0 at the small size is still stuck. Mechanisms that fire only on ghost hits are too rare at 16384 to move it, because the ghost there is short and reuse-gated. v003's gain showed up at the large size instead, the opposite of what it predicted.
- Timing gave the same lesson again. Policy-only diffs showed noise-level shifts. The one structural cost, v002's multi-pop eviction loop, clearly inflated the `put` tail: any eviction path whose work per miss is unbounded is a tail-latency risk.
- Per-entry metadata stayed flat. Both new bits fit into existing padding or tag space.

## Open questions

- Would a stricter ghost→main admission rule for small-demotee ghost hits (for example re-admitting to small instead of main) close web_0's large-size gap? That would be the opposite direction from v003.
- Does the `freq > 1` threshold, or a main-eviction lever that acts on every sweep, move hm_0 at 16384 where ghost-path changes cannot?
- Could v003's large-size hm_0 gain be kept without the web_0 cost, for example by applying the credit only when the ghost hit comes soon after eviction?

## Continuation

Keep cycle-5 v002 as the parent. No cycle-6 variant replaces it. Drop small-queue resizing and any extra rounds inside small, since probation length is ruled out. For web_0, test the ghost→main admission rule as a single variable, preferring stricter admission over looser. For small-cache hm_0, try levers that act on every main eviction, not on ghost hits. The ghost-index shrink for the `put` tail is still untested. Keep eviction work bounded per miss, and give no timing credit to policy-only diffs.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant made one targeted change. v001 used stricter ghost→main admission: small-demotee ghost hits went back into small with `freq = 1`. v002 halved main freq on each sweep instead of decrementing it, built on cycle-6 v003. v003 replaced the ghost deque with a fixed `main_size` ring. None beat the parent (cycle-5 v002). v001 and v002 lost summary hit rate, v003 kept hit rate flat but made the `put` tail worse, and v001 also inflated `put` latency. All three are Winners by rank only.

## Patterns

- The ghost handoff is ruled out as the lever for web_0 at 131072. Looser admission (cycle-6 v003) and stricter admission (v001) both regressed that cell. On this parent, a small-demotee ghost hit already confirms reuse. Forcing another probation pass sends long-reuse-distance keys round the ghost → small → ghost loop and takes small slots from new arrivals. v001's only gain came at web_0 16384, where main turns over fast.
- Faster aging in main hurts. Halving evicts high-freq keys whose inter-hit gap is longer than a sweep, and hm_0 at 16384, the cell it targeted, fell. Together with cycle 5's MAX_FREQ gain, this suggests strongly reused keys need more tenure in main, not less. The "dead residency" theory did not hold up.
- The `put` tail is set by hash probes per miss, not by occasional O(n) cleanup. The ring's constant per-insert liveness probe cost more than the parent's rare, amortized `retain`/scan. v001's extra `evict_small` cascade and ghost churn also raised the tail. Changing the shape of the ghost container is not the lever.
- Per-entry metadata stayed flat again. All diffs fit inside the existing 24-byte entry.

## Open questions

- After probation length, small share, ghost bound and both directions of ghost admission have all failed, is web_0's large-size gap simply out of reach for this policy family?
- Would small-cache hm_0 respond to giving strongly reused keys more retention, for example protecting keys at the freq cap, or to a small/main split that depends on cache size?
- Can the miss path drop a hash probe, for example by tracking ghost liveness without a separate `ghost_index` lookup or by storing fingerprints?

## Continuation

Keep cycle-5 v002 as the parent. Drop all ghost-admission variants, faster freq aging, and ghost container reshaping. For hm_0 at 16384, test levers that extend main tenure for highly reused keys, or a split that depends on cache size. Treat web_0 at 131072 as low priority unless a genuinely new mechanism appears. For timing, target fewer hash probes per miss. Keep eviction work bounded, and give no timing credit to policy-only diffs.


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
