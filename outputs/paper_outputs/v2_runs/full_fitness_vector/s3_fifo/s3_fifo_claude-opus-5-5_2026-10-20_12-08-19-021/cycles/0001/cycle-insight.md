# Cycle 0001 Insight

## Cohort outcome

Three baseline-parented hypotheses attacked hit-rate on workload-a through different levers: a looser small→main graduation rule (H01), ghosting main-queue victims (H02), and ARC-style adaptive small/main sizing (H03). Only H03 (v003) was evaluated and it won. v001 (H01) and v002 (H02) were rejected because the simulator build failed with HTTP 429s from Maven, not because of anything in their code, so their levers are still untested.

## Patterns

- Changing queue *capacity* (how much room small and main get) and leaving the per-key movement rules alone improved hit-rate. So the fixed 25/75 split is not optimal for workload-a.
- No dimension got worse. Per-entry metadata stayed flat, which means size adaptation costs nothing per entry.
- Latency got better too, but nothing in the change explains it. It may just be fewer miss scans because the hit-rate went up, or it may be noise.
- The two resize triggers are lopsided ("main full on graduation" fires almost all the time in steady state). The split probably settles at a balance point set by the clamps and the ghost-hit rate, rather than truly adapting.

## Open questions

- Is the v003 gain coming from the adaptation itself, or just from landing on a better fixed split? The next variant should log or report where `small_size` settles.
- Was the latency improvement real? It needs re-measuring.
- Do H01 (graduation threshold) and H02 (ghosting main victims) help on their own? Both are untested only because of infrastructure failures.

## Continuation

Re-submit H01 and H02 essentially unchanged, preferably with v003 as the parent, since both are movement-rule changes that should stack with v003's sizing lever. Probe the H03 mechanism as well: try a fixed split near where `small_size` settles, or make the grow-small trigger fire less often. Don't credit adaptive sizing for latency gains until they are reproduced.
