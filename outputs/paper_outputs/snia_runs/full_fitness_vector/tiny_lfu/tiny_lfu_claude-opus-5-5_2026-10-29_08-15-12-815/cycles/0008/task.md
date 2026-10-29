# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two variants targeted latency and one targeted accuracy. v001 ported cycle 0006 v003's sketch prefetch and probe-before-increment reorder onto the merged line. v002 prefetched list neighbours ahead of `on_hit` promotion. v003 removed the duplicate sketch increment when a missed `get` is followed by `put` of the same key. All three are Winners by name only. None hit its main target. v001 barely eased get p99, possibly within noise. v002 regressed the tails. v003 gained hit-rate at the small size, lost more at the large size, and regressed latency across the board.

## Patterns

- Software prefetch has run its course on this line. With `increment_hashed` inlined, the sketch-block prefetch overlaps with the inlining gain instead of adding to it: get p50 stayed flat and put picked up a little extra work. Neighbour prefetch can't break the pointer chase, because `prev`/`next`/`region` live inside the hit node, which has to be loaded first. Both prefetch attempts leave hit-rate bit-identical, so hit-rate is no help in telling them apart. They live or die on latency, and both came up short.
- The get→put double count on a miss is not just an overcount. It also does two jobs the policy relies on: it speeds up the aging clock on miss-heavy traces, and it gives newcomers a small edge against the strict `>` admission test. Removing it helps the small size, where the window turns over fast, and hurts the large size, especially web_0. Small and large sizes again want opposite things, the same split seen in cycles 0004–0005.
- The idea that the large size needs better frequency evidence holds up, but v003 shows the evidence lever is tied to aging rate and tie-breaking. Any change to how often counts are added also changes how often the sketch resets.
- Unexplained latency swings on near-trivial diffs are back (v003). Per-op-time verdicts from single runs stay suspect.

## Open questions

- If v003's dedup is kept but skipped increments still count toward `additions`, or `SAMPLE_FACTOR` is lowered, does the small-size gain survive without the large-size loss?
- Would letting ties go to the candidate, or giving it a small explicit bonus, recover web_0 at the large size? That would separate the recency-bias effect from the aging effect.
- Is v003's latency regression caused by the `get` restructuring (`?` → `match` writing `self`), or is it noise?
- Can get p99 be cut by keeping link/region data next to the map value, so promotion loads start without first loading the node?

## Continuation

Drop software-prefetch variants, both sketch and neighbour, on the inlined merged line. Keep cycle 0006's v001 line as the base, since v001 here is effectively equal to it. Rework v003's dedup into a two-variable split: keep the aging clock unchanged (count skipped increments toward `additions`), then test the admission tie-break separately. Aim for the small-size gain without the large-size loss. If tail latency is pursued, change where link/region data lives in memory instead of adding hints, and confirm every latency verdict with interleaved runs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
