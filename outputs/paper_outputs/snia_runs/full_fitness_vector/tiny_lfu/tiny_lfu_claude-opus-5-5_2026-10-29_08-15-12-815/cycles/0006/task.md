# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
