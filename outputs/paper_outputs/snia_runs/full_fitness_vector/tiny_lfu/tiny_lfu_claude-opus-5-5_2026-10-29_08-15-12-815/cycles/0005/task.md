# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
