# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched off cycle 0004's dominating 50%-ghost
incumbent and, for the first time, probed ghost *mechanics beyond length*:
trimming the window below 50%, changing the seed frequency of re-admitted
keys, and changing what populates the ghost. The only genuine hit-rate
lift came from sourcing the ghost from main-queue victims (a thin,
marginal gain); trimming to 3/8 was a latency-for-hit-rate trade, and the
freq=1 seed regressed on both axes.

## Patterns

- The 50% ghost length is confirmed as the hit-rate knee, not a way-point:
  going narrower (3/8) sheds recoverable near-reuse for a shorter put
  membership scan — a latency/quality trade, not a strict win. The
  re-admission peak sits *at* 50%, not below it.
- The graduation-vs-readmission asymmetry is load-bearing, not a bug.
  Granting re-admitted ghost keys an inherited second-chance pass (freq=1)
  over-protects keys whose reuse already happened, crowding main and
  hurting *both* hit-rate and put p99. Re-admitted keys must re-prove from
  freq=0.
- A new, productive axis surfaced: **what populates the ghost**, not just
  how long it is. Recording proven-but-aged main victims (alongside
  small-queue demotions) buys a small hit-rate gain at fixed window length
  — the first ghost-content change tried in any cycle. The lift is thin,
  implying workload-a has only a slim tail of reused-then-aged main
  residents.
- Latency continues to track exactly one scan: the ghost membership walk on
  put. Shortening it (3/8) helps; keeping the ghost fuller via main-victim
  churn costs a touch of put/get latency. The resident rotation is still
  not the driver.
- Metadata discipline held — every change reused the existing ghost deque
  or retuned a constant; no struct state added.

## Open questions

- Is the main-victim ghost source still net-positive once its latency
  churn is weighed against its thin hit-rate gain, or is it a wash?
- With length pinned at 50%, seed-frequency settled at 0, and source now
  including main victims, is there any ghost knob left that lifts hit-rate?
- Is workload-a now effectively hit-rate-bound at this capacity, with only
  latency-side scan trims remaining?

## Continuation

Treat the ghost *length* sweep as done — 50% is the confirmed knee and
going narrower only trades hit-rate for latency. Drop seed-frequency
experiments: re-admitted keys re-proving from freq=0 is load-bearing, so
do not grant inherited survival credit. The live thread is ghost
*content*: the main-victim source gave the only hit-rate gain, so probe
whether it survives a latency/quality reckoning and whether re-admission
candidate selection can be sharpened further. Keep avoiding struct-state
additions, and keep scoring per-op-time only against the ghost membership
scan, never the resident rotation. Expect hit-rate gains to be marginal —
this workload at this capacity may be near its hit-rate ceiling.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched off cycle 0005's dual-source-ghost incumbent
and, for the first time, attacked latency through **scan mechanics
themselves** rather than tuning ghost length or content. The two
median-latency wins landed — an O(1) ghost membership index and a
resident scan reordered to hit main first — while one more ghost-length
widening confirmed the 50% knee still holds and bought nothing. All three
were Winners, but the productive moves were both pure latency plays with
hit-rate held bit-for-bit constant.

## Patterns

- The hit-rate frontier is now treated as closed: every change this cycle
  was explicitly hit-rate-neutral (identical key sets) or a failed
  hit-rate probe. The last ghost-length step (1/2 → 5/8) recovered no
  near-reuse hits even under the doubled dual-source inflow, retiring the
  "re-find the knee for the new content regime" thread — content, not
  length, was the binding constraint, and content is now also tapped out.
- Latency is decomposable into distinct scans, and each yields to a
  *structural* fix rather than a length trim. The ghost membership walk in
  put falls to an O(1) container-level hash index; the resident walk's get
  p50 floor — stuck for many cycles — finally moves once main is scanned
  before small. Both confirm the long-suspected scan drivers by eliminating
  them.
- Every median win redistributes cost to the tail. The hash index trades a
  small predictable per-element scan for rarer hashing/rehash spikes (worse
  put p99); the scan reorder pushes the rarer small-resident, miss, and put
  lookup paths to the tail. p50/p99 are now pulling apart on the same knob.
- Metadata discipline held in spirit but stretched: the scored `Entry` size
  stayed 24 bytes, yet the O(1) index grows the (unscored) container — a new
  category of cost that sits outside the scored summary.

## Open questions

- Are the median latency wins net-positive once the tail regressions
  (put p99 hashing spikes, reordered p99 walks) are weighed — or does the
  scored summary reward p50 enough that the tail cost is invisible?
- Is there a structure that gives O(1) membership *without* a parallel hash
  set's tail spikes (e.g. reusing the resident index, or a cheaper probe)?
- With hit-rate closed and the two main scans addressed, what scan or path
  is left to shorten — is workload-a now at both a hit-rate and a
  median-latency floor?

## Continuation

Treat the ghost frontier — length **and** content — as fully closed: 50%
is confirmed terminal and no widening recovers hits, so stop re-probing it.
The live axis is now **latency via structure, not tuning**: the O(1) index
and the main-first scan reorder both moved long-stuck floors, so pursue
further scan-structural wins. But guard the tail explicitly — both winners
shifted cost into p99, so the next step should look for an O(1) membership
that avoids the hash-resize spike, and weigh whether reorder/index wins
survive a p99 reckoning. Keep the scored `Entry` at 24 bytes, but note that
unscored container growth is now in play. Expect hit-rate to stay flat —
this workload is hit-rate-bound at this capacity, and the remaining game is
how cheaply the same hits are served.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
