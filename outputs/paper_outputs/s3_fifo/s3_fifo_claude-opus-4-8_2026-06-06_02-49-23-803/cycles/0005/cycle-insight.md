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
