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
