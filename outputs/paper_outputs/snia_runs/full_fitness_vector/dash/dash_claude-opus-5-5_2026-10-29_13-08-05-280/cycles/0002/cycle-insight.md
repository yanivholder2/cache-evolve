# Cycle 0002 Insight

## Cohort outcome

Two variants built on cycle 0001's stash reuse gate. v001 removed the gate's extra parts (the per-hit counter writes and the victim comparison), and v002 added reuse-aware MRU landing for demotions. The third, v003, added a reuse flag to v002's free-slot placement. The Winners are v001, which got a faster get path for a small loss of large-cache hit-rate, and v003. v003's own insight reports no improvement on any dimension, so its place among the Winners should be read with caution (see Open questions).

## Patterns

- The cycle-0001 gate splits into separate parts. The two-hit stash threshold gives almost all of the hit-rate gain. The counter writes on normal-tier hits cause most of the median get cost. The victim comparison buys a small large-cache hit-rate gain, mainly on hm_0, for a small get-latency cost. Each part can now be priced on its own.
- Once the gate is in place, other "protect reused items" mechanisms add little. v002's MRU demotion landing helped only slightly, and only where the gate had not already saturated the benefit (web_0 large). v003's flag-limited free-slot placement left hit-rate unchanged in every cell. Extra reuse evidence mostly duplicates what the gate already captures.
- Any reuse check on the common target-hit path costs a little median get, whether it is a counter read, a compare or a conditional store. The summary latency statistic is sensitive to this, so the common hit path should stay free of bookkeeping.
- Large cache sizes are where the remaining hit-rate is won or lost. Both the victim comparison's benefit and the cost of dropping it showed up there, not in the small-cache cells.
- The per-op-time benchmark looks too noisy to separate small changes. v003 regressed by more than its code change can plausibly explain, and that casts doubt on v002's cycle-0001 latency gain.

## Open questions

- Is the per-op-time measurement stable? Re-running an identical variant would show how large the noise is before more latency conclusions are drawn.
- Can a cheaper victim guard win back v001's lost large-cache hits without per-hit counter writes? For example, a single-bit check, or a guard active only at large sizes.
- Why is put p99 higher in v001? Is it from the extra stash churn of unconditional promotion into a full target, and can it be reduced?
- If free-slot placement does not drive hit-rate, what limits web_0 at the large size?

## Continuation

Use v001 (threshold-only gate, lean hit path) as the new baseline. Next, try a cheap, write-free victim guard, possibly only at large cache sizes, to recover hit-rate there. Drop the free-slot placement line (from cycle-0001's v002) and MRU demotion landing, since both are redundant with the gate. Before trusting small latency differences, check measurement noise, for example with a re-run of the baseline.
