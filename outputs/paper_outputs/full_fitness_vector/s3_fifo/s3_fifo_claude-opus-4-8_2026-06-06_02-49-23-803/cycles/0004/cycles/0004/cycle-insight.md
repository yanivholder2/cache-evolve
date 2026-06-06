# Cycle 0004 Insight

## Cohort outcome

All three variants branched off cycle 0003's narrow-split incumbent and
probed the two now-active axes — the latency/hit-rate ghost frontier and
the residual hit-rate levers. The standout was a ghost cap at 50% of main
that *strictly dominated* the parent (faster put **and** higher hit-rate);
the freq-ceiling stack added only a marginal hit-rate sliver, and a
bounded second-chance loop bought no tail win while costing a touch of
hit-rate. All three landed as Winners, but only one moved the frontier.

## Patterns

- The ghost frontier has a clear knee at ~50% of main: the 100% window was
  paying scan cost for reuse-distance coverage workload-a never used, and
  25% over-trimmed legitimate near reuse. The 50% point banks the
  put-latency win *and* recovers (even slightly exceeds) the parent's
  hit-rate — a rare non-tradeoff move. The recoverable near-reuse hits live
  in the 25–50% band of recent evictions.
- The hit-rate grid (split × freq-ceiling) is essentially saturated:
  combining both levers at their best settings compounded only at the
  margin, the smallest hit-rate move yet. No new regime was unlocked; this
  was the last untested cell.
- Bounding the second-chance rotation loop is a cure for a pathology this
  trace lacks: the put tail was never driven by long main-queue rotations
  (the cap almost never fires), so it shed reuse-proven `freq>0` keys for no
  p99 gain. Latency wins on workload-a come from the **ghost membership
  scan**, not the resident rotation.
- Metadata discipline held again — every change retuned a constant or
  threshold; no struct state was added.

## Open questions

- Is the dominating point exactly 50%, or does a setting between 25% and
  50% bank even more latency while holding the hit-rate gain?
- With the hit-rate grid saturated, is there any lever left that lifts
  hit-rate, or is workload-a at this capacity now hit-rate-bound?
- Are there other linear scans (beyond the ghost) whose length could be
  trimmed for latency without touching hit-rate?

## Continuation

The ghost length is the live, productive axis: the 50% cap dominated, so
sweep the 25–50% band to pin the exact point that keeps the hit-rate gain
while maximizing the latency win. Treat the split × freq-ceiling grid as
**done** — it is saturated and further stacking is noise. Drop
rotation-bounding schemes: the put tail isn't rotation-driven on this
trace, so they only shed proven keys. Keep avoiding struct-state additions,
and continue scoring per-op-time only when a change shortens an actual scan
(ghost membership), not the resident rotation.
