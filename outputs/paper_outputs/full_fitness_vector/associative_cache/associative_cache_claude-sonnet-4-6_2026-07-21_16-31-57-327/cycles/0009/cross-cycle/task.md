# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted independent axes — eviction policy semantics, bucket geometry, and struct metadata size — and each moved its primary metric in the right direction, yet none cleared the Winners bar because every variant regressed per-op-time, the shared blocking dimension.

## Patterns

- **Hit-rate gains are available but latency-gated.** Both the LFU-semantics variant and the wider-bucket variant improved hit rate on the LIRS trace, but both paid for it through linear scans: LFU requires scanning for min/max frequency counters, and wider buckets increase the number of entries touched per lookup. The hit-rate signal is real; the bottleneck is eviction/lookup cost.
- **Struct shrinkage does not imply speed.** Removing the unused timestamp field achieved the predicted metadata-size reduction precisely, but the changed memory layout altered cache-line packing enough to raise get latency — showing that struct-size and runtime-performance objectives can conflict even when the removed field is never read.
- **Per-op-time was the universal disqualifier.** No variant avoided a latency regression; it was the single axis that prevented all three from winning, regardless of which primary metric they were optimising.

## Open questions

- Can a constant-time LFU structure (e.g., a frequency-bucketed doubly-linked list) recover the hit-rate win from v001 without the O(n) scan cost?
- Do the hit-rate strategies (LFU semantics and wider buckets) compose — i.e., would combining them produce an additive hit-rate gain or introduce new interference?
- Is the cache-line regression in v003 layout-dependent (fixable with explicit alignment) or inherent to the smaller struct size at this bucket occupancy?

## Continuation

The next cycle should treat per-op-time as a hard constraint and pursue the hit-rate gains already demonstrated — specifically, explore a constant-time eviction structure that preserves LFU's scan-burst resilience while keeping get and put on the O(1) path. Wider-bucket geometry (32×64) remains a promising complement once the latency cost is controlled, and the metadata-size win from struct trimming is worth retaining as a no-regress starting point rather than re-litigating it.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted distinct axes inherited from Cycle 0001's open questions — constant-time LFU ordering, O(1) swap-based promotion, and cache-line alignment via padding — yet none cleared the Winners bar: the LFU bubble variant still regressed per-op-time on the get path, the swap variant hurt both hit rate and latency simultaneously, and the alignment variant recovered tail latency at the cost of a partial metadata-size give-back.

## Patterns

- **LFU hit-rate signal survives; the get-path overhead does not.** The bubble-sort LFU again demonstrated frequency-based eviction's advantage on scan-heavy traces, and eviction is now genuinely O(1) (tail removal). The remaining bottleneck is unconditional loop entry on every `get`, even when the accessed item's rank is already stable — a guard comparing the counter to its left neighbour before entering the loop is the natural next step.
- **Swap-to-back is a double loss at small associativity.** Replacing `remove+push` with `swap` failed on both dimensions: the bounded linear shift is not the latency bottleneck at small bucket sizes, and the ordering approximation degrades hit quality on the Zipfian workload. This axis should be abandoned.
- **Cache-line alignment is a confirmed, targeted fix for p99 spikes.** Padding the compact 24-byte struct to 32 bytes eliminated tail-latency straddling precisely as predicted, halving p99; the cost is 8 bytes of per-item footprint. At 32 bytes the struct still sits below the 40-byte baseline, making alignment padding the right structural foundation going forward even though it does not satisfy the metadata-size metric on its own.

## Open questions

- Can a neighbour-check guard (`if counter > left_neighbour.counter { bubble }`) on the LFU get path reduce unconditional overhead enough to clear per-op-time while preserving the hit-rate win?
- Does composing the 32-byte aligned struct with the guarded LFU bubble produce additive gains on both per-op-time and hit rate, or does the wider item footprint introduce new pressure that offsets the alignment benefit?
- Is there a way to reclaim the metadata-size dimension (currently 32 bytes vs. a 40-byte baseline) while holding the alignment fix — e.g., shrinking a different field rather than adding padding?

## Continuation

The next cycle should compose the two most promising pieces: the 32-byte aligned struct (confirmed p99 fix, already below baseline footprint) as the structural base, and the LFU bubble eviction with a stability guard that skips the bubble loop when the accessed item already outranks its left neighbour. This combination targets per-op-time and hit rate simultaneously without reintroducing the swap ordering regression. The swap-to-back promotion approach should be dropped entirely — it is not the latency bottleneck at these bucket sizes and actively harms hit quality.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the two open threads from Cycle 0002 — reducing get-path overhead via early-return guards and improving hit-rate quality via frequency-aware ordering — plus one structural experiment with two-zone partitioning. No variant reached the Winners bar: the position-0 guard traded latency for hit-rate, the two-zone partition regressed on both axes, and the single-swap LFU gained hit-rate without moving per-op-time.

## Patterns

- **Position-0 guard confirms the latency bottleneck but over-constrains promotion.** The early-return for already-hot items halved p99 latency as predicted, validating the write-elimination hypothesis. However, capping promotion to one swap per access prevents mid-bucket items from converging to the front quickly enough on a Zipfian workload, eroding the hit-rate the parent's full bubble had built. The useful insight is directional: the guard itself is sound, and the cap just needs to be wider than one.
- **Single-swap LFU with the aligned struct is a clean hit-rate gain at no latency cost.** Composing the 32-byte struct from Cycle 0002 with a frequency-ordered single swap lifted hit quality by steering eviction toward genuinely cold items without touching per-op-time — confirming that LFU ordering's advantage is eviction-candidate quality, not struct overhead, and that the aligned layout is a neutral-to-positive base.
- **Two-zone partitioning is a definitively closed avenue.** The hot/cold boundary shifted with every insertion, pushing boundary items back into the cold zone and degrading hit rate, while the hot-zone bypass saved no real work because baseline LRU already keeps hot items where removal is O(1). No reformulation of this design is worth pursuing at these bucket sizes.

## Open questions

- Can a bounded bubble loop (cap of 2–3 swaps) recover the hit-rate convergence that the single-swap variants sacrifice, while still short-circuiting the rare long-distance promotions that drive latency on a Zipfian workload?
- Does composing the position-0 guard with a bounded loop (rather than one swap) produce additive wins on both per-op-time and hit-rate, or does even a short loop reintroduce enough overhead to nullify the guard's p99 improvement?

## Continuation

The next cycle should compose three confirmed pieces: the 32-byte aligned struct as the structural base, the position-0 early-return guard for already-hot items, and a bounded promotion loop capped at 2–3 swaps rather than exactly one. This triplet directly addresses the failure mode identified in v001 (single-swap too conservative) while preserving v003's hit-rate quality and v001's tail-latency benefit. Two-zone partitioning should not be revisited. If per-op-time still does not improve after bounding the loop, the next hypothesis should examine whether the measurement floor itself masks the guard's contribution rather than adding further structural complexity.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants composed pieces recommended by Cycle 0003 — a bounded promotion loop, an unconditional recency advance, and a two-tail eviction selector — with each producing a hit-rate lift but none clearing the Winners bar, primarily because per-op-time remains pinned at the benchmark's timer-resolution floor regardless of approach.

## Patterns

- **Three-swap cap resolves the single-swap bottleneck cleanly.** Extending the promotion budget to three swaps lets mid-bucket items reach the head zone without per-op-time cost, because the early-break on a failed swap means the common Zipfian hot-key case exits after one iteration. Cycle 0003's single-swap cap was the precise constraint; three swaps is sufficient to close the convergence gap.
- **Unconditional recency promotion beats frequency-gated promotion on this workload.** Removing the `lfu_counter` neighbour check and always advancing one step outperformed soft-LFU ordering: the frequency gate was stranding recently-reused items below cold neighbours, and always-advance keeps the tail genuinely cold for the eviction path.
- **The 32-byte aligned struct is a non-negotiable base.** Dropping the compact layout in v003 immediately inflated per-item footprint and likely reintroduced cache-line straddling, erasing the two-tail eviction benefit. Every future variant must carry the alignment fix forward explicitly.
- **The per-op-time measurement floor is now a visible ceiling.** Both v001 and v002 show no latency regression, but neither shows improvement; v002 explicitly attributes this to timer resolution. The benchmark's floor is masking whether any latency gains are real, which means further per-op-time optimisation hypotheses cannot be evaluated until timer resolution improves.

## Open questions

- Do the 3-swap loop (v001) and the always-advance rule (v002) address the same mid-bucket convergence bottleneck from different angles, or are they additive when composed?
- Is the two-tail eviction comparison (v003) worth re-testing in combination with the compact struct, and does it compose cleanly with either promotion strategy?
- Is the 100ns timer floor a benchmark configuration issue, or does meaningful latency differentiation require a workload with higher operation count / longer trace to amortise measurement noise?

## Continuation

The next cycle should compose the 32-byte aligned struct with the 3-swap bounded loop (v001) and test whether replacing the frequency gate with the unconditional always-advance rule (v002) produces additive hit-rate gains or converges to the same outcome. The two-tail eviction selector is worth a clean re-test paired with the compact layout. If per-op-time results remain flat after these compositions, the cycle after that should explicitly investigate whether the measurement floor is the blocker rather than introducing new structural changes — resolving this ambiguity is now more urgent than adding further promotion complexity.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed promotion strategy and frequency-signal placement: one extended the unconditional swap budget, one added a frequency overlay at the eviction boundary, and one gated promotion distance on an access counter. Only the purely positional, write-free approach (v001) landed in Winners.

## Patterns

- **Unconditional multi-swap promotion wins cleanly at no latency cost.** The 3-swap loop's early-break keeps the hot-path at one iteration for already-head items, so Zipfian workloads see the full convergence benefit with no measurable overhead. Promotion budget expansion from one to three swaps was the right lever; the convergence lag at single-step is real and now resolved.
- **Frequency signals on the get path carry consistent tail-latency risk.** Both variants that wrote `lfu_counter` per access surfaced p99 pressure — v002 from counter writes hurting eviction ordering, v003 from the combined weight of counter writes plus up to three additional swaps. The write itself, not the conditional logic, appears to be the cache-line liability on the get path.
- **Frequency overlays at the eviction boundary degrade hit rate on this workload.** Two-tail frequency comparison (v002) overrode clean positional recency, ejecting recently-promoted items before genuine LRU victims. Strict tail eviction remains the correct policy; no frequency signal applied at eviction time has improved upon it across two cycles of testing.

## Open questions

- Does promotion budget beyond three swaps (e.g., four or five) continue to gain hit rate, or have diminishing returns already set in at three given the Zipfian concentration of hot keys near position 0?
- v003's p99 doubling is the first latency signal to clearly clear the measurement floor — does it indicate a general threshold on per-get write volume, and can that threshold be quantified to bound future frequency-integration experiments?

## Continuation

The next cycle should build on v001's 3-swap unconditional loop as a confirmed base and investigate whether frequency information can be incorporated without any write on the hot get path — for example, sampled or epoch-based counters that update only on eviction or put, not on every get. Two-tail eviction and any per-get counter write should be treated as closed unless a fundamentally different encoding is proposed. If hit rate appears to have plateaued after further promotion-budget trials, the focus should shift to understanding the measurement floor more precisely, as v003's p99 signal suggests latency differentiation is now observable under write-heavy get paths.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort probed three independent axes: promotion-budget saturation, bucket-count scaling, and frequency-informed eviction via a write-once binary flag. Both winners stayed on the 3-swap positional base — one confirming its ceiling, the other pairing it with a second-chance tail scan that measurably improved hit rate.

## Patterns

- **3-swap promotion budget is confirmed optimal; the ceiling is now mapped.** Extending to five swaps produced a slight hit-rate regression, meaning items beyond three hops from the head are accessed rarely enough on this Zipfian workload that forcing them forward displaces items that cycle through the top positions. The useful range of promotion-budget exploration is exhausted.
- **Second-chance eviction with a write-once flag is the most promising hit-rate lever found so far.** A binary "ever-accessed" marker — written once at first touch, never again on the hot get path — lets the eviction scan skip past genuinely warm items and eject cold newcomers instead. This directly answers Cycle 0005's call for frequency information without per-get writes.
- **The latency regression in v003 traces to the 3-swap loop, not the flag.** The second-chance scan itself is inexpensive; it is the unconditional position-update swaps on every hit that push per-get write cost above the baseline. The hit-rate gain and the latency cost are therefore separable — they come from different mechanisms.
- **Halving bucket count is a definitively closed direction.** Fewer buckets raise collision density faster than added depth can absorb, and the 3-swap budget cannot rescue items stranded beyond position three in doubled-width buckets. The hit-rate regression replicates Cycle 0001's findings and removes any remaining ambiguity.

## Open questions

- Can the second-chance eviction scan be paired with a cheaper promotion strategy (fewer swaps, or none at all) to preserve the hit-rate lift while eliminating the per-get write regression?
- Does a zero-swap get path — pure positional read with no item movement — combined with the second-chance eviction scan still outperform strict LRU, or does recency ordering degrade too quickly without any promotion?

## Continuation

The next cycle should decouple the two mechanisms in v003: keep the write-once binary flag and second-chance tail scan, but reduce or eliminate the per-get promotion swaps. Specifically, try pairing the second-chance scan with (a) a single swap only — recovering the position-0 guard pattern from Cycle 0003 — and (b) no promotion at all, relying entirely on eviction quality rather than positional convergence for hit-rate. If either pairing holds hit rate near v003's level while restoring latency to the v001 baseline, it becomes the new confirmed base. Bucket count and promotion budget beyond three swaps should not be revisited.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants attempted to decouple the second-chance eviction scan's hit-rate benefit from the 3-swap promotion cost by reducing swaps (to one, then to zero) or by tightening flag placement; none cleared the Winners bar, and v001 was selected as least-bad among a cohort that uniformly regressed.

## Patterns

- **Second-chance scan and 3-swap promotion are mechanically coupled, not independent.** Reducing to one swap or eliminating promotion entirely caused hot items to age into the scan window alongside cold ones, stripping the scan's ability to distinguish them. The eviction quality gain seen in Cycle 0006 is not a property of the flag alone — it requires promotion to maintain hot-item separation from the cold tail.
- **The 3-swap write cost is already below the 100 ns measurement floor.** Eliminating promotion swaps entirely bought no measurable per-op-time improvement, which means the latency regression attributed to "3-swap write pressure" in Cycle 0006's v003 originated elsewhere — the hypothesis that swaps were the bottleneck is now falsified.
- **Flag placement (first-touch vs. position-0 arrival) is neutral on this workload.** Once-accessed mid-range items were not a meaningful drain on eviction capacity under the 8-slot scan window; tightening the threshold produced no measurable hit-rate movement.
- **Compact 32-byte struct loss remains a recurring failure mode.** Variants that overlay only the eviction policy file without explicitly carrying forward the aligned item layout reintroduce per-item footprint bloat and put-path latency — this has now surfaced in multiple cycles and must be treated as a required invariant, not an incidental fix.

## Open questions

- If 3-swap write cost is below measurement resolution, what actually caused the put p99 regression in Cycle 0006's v003 — could it have been an artifact of a different structural change in that variant, such as a scan-window size or a metadata layout detail?
- Is there a write-free way to maintain the hot-item / cold-item separation that the 3-swap loop currently provides, or is positional movement strictly necessary for the second-chance scan to function?

## Continuation

The decoupling hypothesis has been tested exhaustively and rejected — the next cycle should abandon further attempts to separate the second-chance scan from the 3-swap loop and instead treat their combination as a confirmed unit. The productive path is to return to the Cycle 0006 v003 configuration (3-swap + write-once flag + 8-slot second-chance scan + compact 32-byte struct) and diagnose the latency regression source directly, since the swap-elimination experiments have ruled out write pressure as the cause. Compact struct inheritance must be enforced in every variant going forward.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants probed the second-chance scan's internal parameters — window depth and scan direction — rather than its interaction with promotion. The two winners confirmed that the 8-slot window is already near-optimal in depth, and that reversing iteration order to evict the oldest cold candidate first is a free hit-rate gain.

## Patterns

- **Scan direction is the highest-value tunable found in the second-chance scan.** Switching from newest-cold-first (LIFO-among-cold, tail position) to oldest-cold-first (LRU-among-cold, inner edge of window) improved hit rate at zero latency cost. The parent's LIFO ordering was silently discarding items too young to have received a second request; LRU-among-cold preserves them long enough to convert.
- **8-slot scan depth is already the sweet spot; neither direction of change helps.** Halving to 4 positions degraded p99 latency without any hit-rate movement — on the rare evictions where cold items don't occupy the bottom 4 positions, the narrower window falls back to a worse candidate and spikes tail latency. Expanding to full bucket depth added unbounded comparison work with zero hit-rate return, confirming that never-accessed items beyond the 8-slot tail are essentially absent under the 3-swap promotion regime.
- **Scan cost is positional, not directional.** The iteration-order flip in v003 carries no measurable overhead because scan length is unchanged; all the per-call work budget is in the traversal count, not the comparison order.

## Open questions

- Is the LRU-among-cold ordering optimal, or would a frequency-weighted tie-break within the cold window (e.g., evict the item with the fewest accesses among the cold candidates) add further hit-rate lift on this workload?
- With scan direction confirmed fixed and scan depth confirmed fixed, is there remaining headroom in the second-chance flag's definition — e.g., expiring or resetting the "ever-accessed" bit after N eviction passes to prevent stale warm markers from permanently shielding aging items?

## Continuation

The next cycle should treat the 3-swap loop + 8-slot second-chance scan + LRU-among-cold eviction order as the new confirmed base, replacing the prior LIFO ordering throughout. The remaining structural unknown from Cycle 0007 — what actually caused the put p99 regression in Cycle 0006's v003 — has not been resolved and should be targeted directly: compare the new base against the original 8-slot LIFO configuration on put-heavy access patterns to isolate whether the regression was a metadata-layout artifact, a scan-fallback artifact, or something else. Flag-lifetime and frequency-within-cold experiments are the natural next frontier if the put regression diagnosis comes back clean.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants attempted to refine the second-chance scan's eviction quality — by adjusting the warm-all fallback index, introducing a two-generation access counter with priority passes, or expiring warm flags on deep-cold displacement — and all three regressed. v003 was selected as least-bad among a uniformly regressing cohort.

## Patterns

- **The base's LRU-among-cold fallback index (`len - 1`) was already correct; the H01 hypothesis had the layout direction inverted.** In this cache's positional model, higher index means less recently used, so `len - 1` is the genuine LRU tail. Changing the fallback to `len - scan_depth` selected the *most recently* inserted item in the cold window — anti-LRU — explaining the hit-rate drop without any algorithmic flaw in the parent.
- **Single-access items carry real reuse probability on a Zipfian workload.** Elevating them as eviction targets (generation-1 priority pass in v002) drained items that would have converted to confirmed-warm items on their next request. The second-chance flag's binary (ever/never) distinction already captures the meaningful split; finer-grained generation counts add classification cost without eviction benefit under this distribution.
- **"Deep-cold position" is not a reliable proxy for "stale warm marker."** On Zipfian access, legitimately warm items accumulate near the cold tail simply through low-but-nonzero access frequency. Expiring flags based on position alone (v003's H03) conflates low frequency with staleness, evicts genuinely reusable items, and introduces an extra conditional store that raises tail latency even on a rarely-firing path.
- **Compact struct omission is still recurring.** v002 silently dropped `item.rs` from its overlay and reverted to the 40-byte baseline layout — the third cycle in a row where this invariant has been violated. Variants that touch only `eviction_policy.rs` must explicitly carry the `item.rs` overlay forward.

## Open questions

- The put p99 regression from Cycle 0006's v003 has never been directly diagnosed; the swap-elimination experiments in Cycle 0007 ruled out write pressure, but the actual cause — metadata layout artifact, scan-fallback behavior, or otherwise — remains open.
- Has the 3-swap + 8-slot + LRU-among-cold + write-once-flag base converged near a local optimum for this workload, or is there a structurally different axis (e.g., adaptive scan depth, insertion-point policy) that hasn't been explored?

## Continuation

The three axes probed in this cycle — fallback-index tuning, generation-counter granularity, and flag expiry on displacement — have all been tested and rejected; the next cycle should not revisit them. The most tractable unresolved thread is the Cycle 0006 put p99 regression: the next hypotheses-writer should design a direct A/B comparison between the current confirmed base and the original 8-slot LIFO configuration on put-heavy access patterns, isolating whether the regression was tied to the LIFO ordering, the compact struct transition, or a scan-fallback edge case. If that diagnosis clears, insertion-point experiments — placing new items somewhere other than the conventional tail position — are a structurally distinct axis worth opening.


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
