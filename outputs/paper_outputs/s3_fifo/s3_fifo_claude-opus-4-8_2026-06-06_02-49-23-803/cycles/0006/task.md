# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched off cycle 0002's narrow-split winner and
probed the *knees* of the levers established so far: pushing the split
one step narrower, stacking the freq-ceiling raise on the narrow split,
and shrinking (rather than widening) the ghost queue. The two
retention/capacity moves lifted hit-rate again but by diminishing
margins; the ghost-shrink traded a small hit-rate loss for a large
put-latency win — and all three landed as Winners, signalling latency is
now scored alongside hit-rate.

## Patterns

- The split sweep is still monotone but clearly flattening: another
  capacity shift into main paid off, yet less than the prior step. The
  under-screening boundary was not crossed — it sits just past the
  current narrowest split, so the knee is near but not found.
- The freq-ceiling and split levers are confirmed orthogonal: deeper
  durability for proven keys still helps when combined with the narrow
  split, because how-long-a-key-survives is independent of how-many-fit.
  But the combined gain is marginal — extra second chances are starting
  to protect keys workload-a no longer reaccesses.
- Ghost length is a **latency/hit-rate tradeoff**, not a free hit-rate
  lever in either direction. Cycle 0002 showed widening it hurts; this
  cycle shows shrinking it also hurts hit-rate (the freshest quarter
  drops legitimate near reuse) while sharply cutting the linear
  membership-scan cost in put. The reuse distance this trace rewards is
  narrow but extends past the most-recent 25% of evictions.
- Metadata discipline held: every winning change retuned an existing
  field or constant rather than adding struct state, keeping metadata-size
  flat.

## Open questions

- Where exactly is the split knee — does the very next narrowing finally
  under-screen one-hit-wonders and regress?
- Is there a ghost length between ~25% and 100% of main that keeps most
  of the latency win while recovering the near-reuse hits lost at 25%?
- With latency now rewarded, are hit-rate and per-op-time pulling in
  opposite directions on the same knobs, forcing explicit tradeoffs?

## Continuation

The hit-rate levers (split, freq-ceiling) are deep into diminishing
returns — take at most one more confirming step on each and expect the
split knee imminently; stop assuming monotonicity holds. The newly
interesting axis is the **latency/hit-rate frontier** the ghost-shrink
exposed: sweep intermediate ghost lengths to find the point that banks
the put-latency win without shedding near-reuse hits. Keep avoiding
ghost *widening* and any change that adds struct state. Treat per-op-time
as a real, scored dimension now — but only when a change actually alters
a scan length (ghost/resident traversal), not as run-to-run jitter.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
