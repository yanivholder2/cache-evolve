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
