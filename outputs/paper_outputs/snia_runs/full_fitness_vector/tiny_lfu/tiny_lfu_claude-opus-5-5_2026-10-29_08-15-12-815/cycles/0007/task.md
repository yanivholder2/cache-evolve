# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.
