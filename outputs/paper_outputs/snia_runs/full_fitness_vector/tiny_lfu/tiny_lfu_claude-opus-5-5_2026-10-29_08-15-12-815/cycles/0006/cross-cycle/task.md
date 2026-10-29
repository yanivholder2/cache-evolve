# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline, and each tested one axis on its own: residency structure (v001, W-TinyLFU window plus SLRU), sketch freshness (v002, periodic halving with capped counters), and hashing cost (v003, a single cheap hash per key). The winners were v001, which gave the clearest hit-rate gain, and v003, which gave the largest per-op-time gain. v002 also improved hit-rate, but it lost ground on median latency.

## Patterns

- The baseline had two separate weaknesses, and they map cleanly to the two winners. Hit-rate was held back by admission policy (strict admit-or-drop, a sketch that never forgets). Latency was held back by repeated SipHash. Neither winner's fix touched the other's axis, so the two should combine with little conflict.
- Both hit-rate fixes (v001, v002) gained most at the small cache size and much less at the large one. Admission contests at small capacity are where the baseline lost most.
- `get` p99 rose in v001 and in v003, for different reasons: extra region branching and relinks in v001, exposed sketch memory misses in v003. v001 and v002 both improved the `put` tail, but neither could explain why, so treat that as weak evidence. The get tail is now the main per-op-time weak spot across the cohort.
- Metadata cost was flat or slightly better, apart from v001's per-node region byte, which the padding turns into a full word.

## Open questions

- Do v001's window/SLRU and v002's aging add together, or does the window already absorb most of the stale-hot-key problem?
- Is v003's get-tail rise real, or benchmark noise? Would a cache-line-local sketch block fix it?
- Can v001's region tag be packed into spare bits to win back the per-node bytes?

## Continuation

Next cycle should start by merging v001's W-TinyLFU structure with v003's single-hash fast path. Then layer v002-style aging on top, stored as packed 4-bit counters so it costs no extra heap or sweep work. After that, target the get tail with a cache-line-blocked sketch, and remove v001's region byte. Do not try aging on its own again over the u32 counter layout. Its latency cost is already known.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants branched from cycle 0001's fast-hash winner. v001 merged in the W-TinyLFU window plus SLRU structure, and v002 replaced the sketch with a cache-line-blocked one using packed 4-bit counters and aging. The third, v003, added an adaptive hill-climbing window on top of cycle 0001's W-TinyLFU. v001 and v002 won because each improved hit-rate and per-op-time together. v003 gained hit-rate only at the small cache size and paid for it in latency on every op.

## Patterns

- The two winners changed separate parts of the cache: v001 changed residency and the map key, v002 changed sketch storage. Both still improved hit-rate and latency together, so the earlier tradeoff between the two axes is gone. Each attacked a different cost that the cycle-0001 fast-hash parent still had: v001 removed the second hash into the map, and v002 removed the scattered sketch cache misses. The get-tail weakness flagged last cycle is resolved in both.
- Most hit-rate gains are still at the small cache size. Each hit-rate mechanism (window/SLRU, aging, adaptive window) also showed a small slip in one large-cache configuration. This is consistent across the cohort: at large capacity, mechanisms that forget or bypass frequency evidence start to cost hit-rate.
- Packing bits paid off twice. Sentinel-tagged links removed the region-tag cost and shrank the node (v001). Nibble packing shrank the sketch heap and made aging cheap (v002). Running aging over packed counters did not bring back the latency cost cycle 0001 saw.
- Runtime adaptivity (v003) cost latency on every op through rebalance work and bookkeeping in `get`. Its hit-rate gain may come partly from the path that fills the window without a frequency contest, not from window size alone.
- There are unmeasured costs: v001 stores an extra hash in each map entry, and v001 and v003 grew the fixed cache struct. The current metadata metric does not capture either.

## Open questions

- Do v001's residency/map-key changes and v002's blocked aging sketch combine additively, as the previous merge did?
- What causes v002's large-cache slip: 4-bit saturation, the fixed sample-period multiplier, or correlated collisions within a block?
- Does v003's small-cache gain survive once the frequency contest is restored on the window-fill path?

## Continuation

First, merge v001 and v002 (window/SLRU with a prehashed map key, plus the blocked 4-bit aging sketch). Then tune the aging sample period, or the counter width, per cache size to fix the large-cache slip. Also apply the `NIL` sentinel encoding to `next`, and count the per-entry map hash in the metadata figures. Do not pursue the hill climber in its current form. Try it again only with the contest restored and a climber that stops at its step floor, and only if the merged variant still leaves a gap at the small cache size.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Two variants branched from cycle 0002's window/SLRU winner. v001 swapped in the blocked 4-bit aging sketch, and v002 shrank the node links to sentinel-encoded u32s. v003 added conservative update to cycle 0002's blocked-sketch winner, which has no window/SLRU. All three are Winners, but each won on a single axis: v001 on hit-rate and metadata, v002 on metadata only, and v003 on hit-rate, where it recovered the large-cache slip. Every variant gave back some per-op-time.

## Patterns

- The sketch increment is now the latency bottleneck. v001 pays for saturation checks and sample counting on every `get`. v003's branchy two-pass conservative update pays much more. The cache-line layout still helps the tail, but per-access control flow, not memory, now sets the median.
- The merges keep adding up. The aging sketch combined cleanly with window/SLRU (v001), just as the cycle-0002 merge did. The compact links left hit-rates bit-identical (v002). Neither change interfered with residency behaviour.
- Cycle 0002's large-cache slip came from correlated in-block overestimation, not the aging period. Conservative update fixed that cell and lifted the others a little without touching `SAMPLE_FACTOR`. This is the cohort's clearest causal result, though so far it holds only on the parent without window/SLRU.
- v002's latency regression has no mechanism in its diff and could be benchmark noise or a codegen artifact. Do not treat "compact links are slower" as a lesson yet.

## Open questions

- Does v002's latency regression reproduce when run back to back with v001 in the same session? If it does, is the cause codegen, such as bounds checks the compiler no longer removes?
- Can a branchless, masked-add conservative update keep v003's accuracy at close to the latency of a single-pass increment?
- Does conservative update give the same large-cache gain on top of window/SLRU, or does the window already absorb some of it?

## Continuation

Merge the three lines: v001's window/SLRU with the aging sketch, plus v002's u32 links, plus v003's conservative update. Implement the update as a branchless per-row mask applied with word-level saturating adds, so `get` stops paying for branches. Re-benchmark the u32 links against their parent in the same run before deciding whether to keep them. Do not tune `SAMPLE_FACTOR` or the counter width for the large-cache slip, because conservative update already explains and fixes it.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Each variant tested one item left open by cycle 0003. v001 put a branch-free conservative update on the window/SLRU aging line. v002 ported the blocked aging sketch onto the u32-link layout. v003 fixed the admission window at 5%. All three are Winners on hit-rate. v002 also won on sketch metadata. None improved per-op-time, and only v002 held it roughly flat.

## Patterns

- The two hit-rate levers cover opposite cache sizes. Conservative update (v001) gains only at the large size, where close mid-range frequencies make small overestimates decide the contest. A larger window (v003) gains only at the small size, where a 1% window cannot hold a recency burst, and it slips slightly at the large size. These look complementary, not overlapping.
- The sketch increment is still the latency bottleneck, but the cause has moved. Going branch-free did not help, because the serial dependency chain through `min` costs more than the well-predicted branches it removed. Shortening that critical path is the lever. Removing branches alone is not.
- The u32-link latency regression held up in a same-run test against the same sketch, so it belongs to the residency layer (link codegen), not to scattered sketch access. This is now an established result, not suspected noise.
- Per-op-time readings are becoming unreliable. v003 slowed down on every op with a byte-identical hot path. Read with cycle 0003's unexplained regressions, this suggests the benchmark's noise floor is close to the size of the effects being measured.
- Merges keep adding up: the aging sketch carried over to the 24-byte node with no interference. Its hit-rate was close to v001's figure but not exactly equal, which is still unexplained.

## Open questions

- Why is v002's hit-rate not identical to v001's when it uses the same copied sketch? Is the copy faulty, or does residency order differ?
- Can a SWAR or pairwise min (or an approximate min) keep conservative update's accuracy at the parent's increment latency?
- Is the u32-link slowdown caused by bounds checks or mask/widen sequences in `unlink`/`push_back`/`on_hit`?
- How much of the recent latency movement is noise? Repeated runs or interleaved A/B would tell.

## Continuation

Merge v001's conservative update with v003's window, but size the window as an absolute floor (for example, max of 1% and roughly 800 entries) so the large size keeps its main-region slots. Then shorten the increment's `min` dependency chain. Also fix codegen in the u32-link paths separately, and check that v002's sketch matches v001's hit-rate exactly. Before trusting any per-op-time verdict, firm up the benchmark with repeated or interleaved runs. Do not pursue branch removal alone as a latency fix, and do not raise the window percentage uniformly.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Each variant took one item from cycle 0004's continuation. v001 rewrote the u32-link residency codegen on the 24-byte line. v002 added a size-aware window floor on top of conservative update. v003 replaced the exact four-way min with a pairwise min. All three are Winners. v001 won on per-op-time and on the cache struct's size, with hit-rate unchanged. v002 won a small hit-rate gain. v003 matched its parent's hit-rate with a cheaper update rule. v002 and v003 both got much slower on per-op-time, and neither diff explains it.

## Patterns

- The u32-link slowdown is fixed, and the cause is confirmed as residency codegen. Keeping indices in u32 throughout, writing tagged links in one store, and indexing without bounds checks made the 24-byte node about as fast as the 40-byte one or faster. Put gained more than get. So the 24-byte layout now has no latency cost, and it is the residency layer to build on.
- The hit-rate levers still add up, but the gains are getting smaller. The window floor leaves the large size bit-identical to its parent and helps only the small size, mostly on hm_0. Pairwise conservative update keeps nearly all of full conservative update's accuracy. The exact global minimum turned out not to matter, so a cheaper approximation is acceptable.
- Cycle 0004's diagnosis that the serial `min` chain sets latency now looks wrong. Shortening the chain did not recover latency, and the chain is too short to have caused the slowdown. The cost seen when conservative update was added is more likely an inlining/codegen effect around `increment_hashed`, or noise.
- The per-op-time measurement is not reliable at this point. v002's hot path is byte-identical to its parent, yet it slowed down on every op, with tails hit hardest. v003's slowdown has the same tail-heavy shape. v001's large speedup ran in the same cycle. Any per-op-time comparison across this cohort may partly reflect host conditions and should be confirmed on a stable host.

## Open questions

- Did v002 and v003 really slow down, or did the host change during the run? An interleaved re-run of v001, v002, v003 and their parents would tell.
- Does `increment_hashed` stop being inlined into `get`/`put` once conservative update is added, in either the full or the pairwise form?
- Why is the window floor's small-size gain specific to the trace (hm_0 gains, web_0 barely moves)?
- Cycle 0004's unexplained hit-rate mismatch between the 24-byte and 40-byte lines is still unresolved, and it needs an answer before the lines are merged.

## Continuation

Merge into one line: v001's u32 residency layer, v002's window floor, and v003's pairwise conservative update. Before the merge, check that the 24-byte line's sketch matches the 40-byte line's hit-rate exactly. Force `increment_hashed` inline (`#[inline(always)]`) or check the assembly, and judge per-op-time only from repeated or interleaved runs against the parent. Do not try more variants that only shorten the min chain, and do not count v002's or v003's latency numbers against them until they are re-measured.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's 24-byte u32-link line. v001 merged in the window floor, pairwise conservative update and `#[inline(always)]`. v002 let each sketch row address the whole 64-byte block. v003 prefetched sketch blocks and moved the map probe ahead of the increment. All three are Winners, but for different reasons. v001 is the only one that gained hit-rate, and it also lowered get p50. v003 won on latency alone (get p50 and the put tail), with hit-rate bit-identical to its parent. v002 is a Winner only by name: hit-rate stayed flat at both sizes and put latency got worse.

## Patterns

- The merges still add up. The window floor and pairwise conservative update stacked on the 24-byte layout without cancelling, and v001 beat its hypothesis's prediction. Keeping v001's odd-count `reset()` did not stop the gain, so the reset is no longer a suspect for the earlier hit-rate gap between the 24-byte and 40-byte lines. That line simply lacked both levers.
- The conservative-update latency cost is settled. Forcing inline removed it and even lowered get p50, so it was an out-of-line-call artifact, not a real cost and not host noise. Latency is now improving through codegen and instruction ordering (v001 inlining, v003 memory-level parallelism), not through changes to the update rule.
- Get p99 has become the one stubborn metric. v001 widened it, likely through a larger inlined body or a different reset rate, and v003's prefetch left it unchanged. The likely remaining source is the `on_hit` node and neighbour-link misses on probation→protected promotion.
- v002 is a useful negative result. Moving counters around inside a fixed block cannot lower load per counter. The fixed row-to-word-pair layout is in fact valuable, because rows that never alias let the four RMWs run independently. Overestimation has to be fought with more counters per addition or with conservative update, not with a different layout inside the block.
- v003's prefetch gain and its inline promotion landed together, so how much each contributed is still unknown. v001 shows inlining alone also lowers get p50.

## Open questions

- Does v003's prefetch/reorder stack on top of v001's merged line, or do the two get-p50 gains overlap because both come from inlining?
- Is get p99 caused by promotion-path link misses? Prefetching the neighbour nodes in `on_hit` would test this.
- Is v001's small hm_0 dip at the large size a real ceiling effect of conservative under-counting, or noise?

## Continuation

Apply v003's sketch prefetch, probe-before-increment reorder and paired `evict_one` prefetch to v001's merged line, and judge it against v001 in interleaved runs. Target get p99 next, for example by prefetching or reordering the `on_hit` promotion loads. Drop in-block addressing changes for good, and keep the row-to-word-pair layout because it rules out aliasing. Any further large-size hit-rate work must change counters per addition, not their placement inside a block.


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
