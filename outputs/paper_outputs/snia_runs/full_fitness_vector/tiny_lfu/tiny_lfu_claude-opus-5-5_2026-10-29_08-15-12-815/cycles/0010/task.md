# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Two variants split cycle 0007 v003's miss-dedup into its two coupled effects: v001 restored the aging clock on the incumbent line, and v002 added a guarded admission tie-break on top of v003. v003 tried a CLOCK reference bit for protected hits to cut tail latency. All three are named Winners, but only v002 is a real one. It raised hit-rate in all four cells and has no hurt entries. v001 came in slightly below its parent on aggregate hit-rate, and v003 regressed both latency and hit-rate.

## Patterns

- The two-variable split worked, and it changes the picture of the dedup. Restoring the aging clock (v001) recovered most of the large-size web_0 loss but turned the small-size hm_0 gain into a loss. Restoring the tie edge (v002) recovered large-size web_0 while keeping and slightly extending the small-size gains. So the strict `>` tie-break, not only the slower clock, caused much of v003's large-size regression. And the dedup alone is not a win: the parent's miss→put double count was an implicit "seen again" bonus.
- A bonus for re-seen keys works best when it is explicit and bounded. v002's `>= 2` guard gives ties to re-referenced candidates and still keeps one-hit wonders out. That serves small and large sizes at once, which resolves for now the small-vs-large split seen since cycle 0004.
- v002's `get`/`put` are byte-identical to its parent, yet every latency percentile improved sharply. This mirrors v003's unexplained slowdown in cycle 0007 in reverse. Per-op-time noise between runs is still large enough to swamp real effects. Cycle 0007 v003's latency regression should now be treated as noise, not as a cost of dedup.
- Bits packed into link words hurt on this layout. The parent's protected-hit relinks were blind stores that the store buffer absorbed. Keeping a flag inside `next` turned every relink into a load-then-store, and the bounded second-chance loop multiplied relinks on promotion. The pointer-chase theory for get p99 has now failed three times (sketch prefetch, neighbour prefetch, and in-link CLOCK).

## Open questions

- Does v002's tie-break on top of the restored aging clock (v001's `tick()`) gain more than either one alone, or does the clock change undo the small-size gain?
- Is the `>= 2` threshold optimal, and would it hold at other cache sizes?
- Where does get p99 actually come from, if not from neighbour-link stalls? A reference bit in a separate slot-indexed bitset would show whether the problem is CLOCK itself or carrying the bit in `next`.

## Continuation

Adopt cycle 0008 v002 as the new base for hit-rate work. Next, test it combined with v001's clock-preserving `tick()`, and sweep the tie-win threshold. Drop dedup without a compensating bonus, and drop any flag packed into link words. If tail latency is pursued again, keep the reference bit outside `Node` and confirm per-op-time only with interleaved runs, since single-run latency verdicts went unreliable for the second cycle in a row.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Two variants tried to stack hit-rate levers. v001 ported the guarded tie-break, with its threshold raised to 3, onto the incumbent double-count line. v002 added the clock-preserving `tick()` to cycle 0008 v002's dedup plus tie-break line. v003 replaced protected move-to-tail with a CLOCK reference bit kept in a separate bitset. All three are Winners by name only. None beat its parent on summary hit-rate. v001 gained in only one large-size cell. v002 lost three cells and gained none. v003 regressed every hit-rate cell and every latency percentile.

## Patterns

- Every bonus for the window candidate fixes the same large-size newcomer loss, so the bonuses overlap instead of adding up. The double count, the tie-break and the restored clock each target that loss. Combining any two adds nothing at the large size and over-favours recency at the small size, where estimates are low, collide often and produce many ties.
- The small-size gain from cycle 0008 v002 comes from slower aging, not from the tie-break. Re-adding the tick took it away. At small capacity, a short aging period is what hurts.
- The idea that get p99 comes from the protected relink is now ruled out. Keeping the bit outside `Node` avoided cycle 0008's load-then-store problem, but the CLOCK policy itself breaks recency order in the protected segment and makes promotion heavier. This is the fourth failed attempt to explain the get tail as a pointer-chase or link-write cost.
- Latency noise from run to run struck for the third cycle in a row. v002 got faster on code that was unchanged or slightly heavier on the hot path. Only v003's consistent regression across all four latency statistics looks like a real effect.

## Open questions

- Does a size-dependent aging rate (slower `SAMPLE_FACTOR` at small capacity) keep cycle 0008 v002's small-size gain while helping the large size?
- Should the tie-break threshold scale with capacity or sketch width instead of being a fixed constant?
- Where does get p99 come from? The inlined sketch update and the map probe are the remaining suspects.

## Continuation

Keep cycle 0008 v002 (dedup, no tick, tie-break `>= 2`) as the hit-rate base. Drop the uniform tick, and drop stacking the tie-break on the double-count line. Probe aging rate, or tie-break thresholds, that depend on capacity. Drop CLOCK or reference-bit replacements for protected LRU entirely. For get p99, profile the sketch update and the map probe instead of the list links, and make no latency claim without interleaved repeat runs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
