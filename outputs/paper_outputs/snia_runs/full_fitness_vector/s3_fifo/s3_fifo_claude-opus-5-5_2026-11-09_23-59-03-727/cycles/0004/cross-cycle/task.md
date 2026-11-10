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
