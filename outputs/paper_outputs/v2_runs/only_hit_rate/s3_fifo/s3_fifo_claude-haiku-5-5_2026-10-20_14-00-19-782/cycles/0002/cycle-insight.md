# Cycle 0002 Insight

## Cohort outcome

The cohort tried a cost-side refactor of ghost-membership lookup (v001), a single-constant change to the main-queue frequency cap (v002), and a freq-layout change (v003) that did not land in the overlay. Both Winners (v001, v002) are hit-rate neutral against the parent; neither shows a demonstrated gain, and v003 showed an unexplained hit-rate drop.

## Patterns

- Hit-rate stayed flat for both Winners. v001 is a pure cost refactor that preserved eviction order, and v002 raising the main-queue frequency cap showed no effect on this trace.
- Neither Winner reported metadata-size or per-op-time, so the cost claims behind v001 are unverified. This is the same gap the prior cycle flagged; it persisted into this cohort.
- v002's null result suggests few main-queue residents on this trace build up enough hits to reach the old cap, so the frequency axis looks inert here, consistent with the prior cycle's sizing and routing findings.
- v003 did not implement its hypothesis (the layout change is absent from the overlay). Its hit-rate drop cannot be attributed to the layout idea and may reflect an overlay or build mismatch rather than the change itself.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the v001 ghost-lookup refactor reduce per-op-time, and at what metadata cost? It cannot be ranked on cost until both are reported.
- Is the v003 drop real, or an artifact of the mismatched overlay? Does the hit-rate drop reproduce on a clean build?
- Does the frequency cap matter at other capacities or workloads, where main-queue residents may accumulate more hits?

## Continuation

Keep the ghost-lookup refactor (v001) only as a cost candidate, and require a rerun reporting per-op-time and metadata-size before treating it as a win. Drop the frequency-cap tweak (v002) on this trace; it has now shown no signal in two cycles. Before the next cycle, verify the v003 overlay and rebuild its layout variant; do not test any layout change until the build is confirmed clean.
