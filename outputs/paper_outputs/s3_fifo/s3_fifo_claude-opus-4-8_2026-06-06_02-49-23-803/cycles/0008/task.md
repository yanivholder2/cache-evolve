# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched off cycle 0006's main-first-scan incumbent and
converged on a single target: the get-p50 floor, the median scan-to-hit
distance that has resisted every length/content tweak. Three distinct
mechanisms were tried — an O(1) hash index over a key-only resident store,
in-place front-loading of saturated keys, and a dedicated short `hot`
segment. Only the hash index actually lowered get-p50; both
reorder-the-linear-scan approaches missed the median and merely shifted
cost to the tail. Hit-rate stayed bit-identical across all three.

## Patterns

- get-p50 is structural, not spatial. The two variants that kept the linear
  scan and tried to make the median hit *shallow* — by clustering hot keys
  at the head or carving them into a short front segment — both failed at
  their stated target. The median get does not land where Zipfian intuition
  predicts: front-loading gave no median savings, and the segment was
  probed-then-missed often enough that the extra hot-before-main scan made
  p50 worse. Only eliminating the walk entirely (one hash probe) moved the
  floor.
- The hash index was the cycle's strict win: it dropped get-p50, get-p99,
  *and* both put percentiles together, because constant-time resolve also
  collapses the miss and update-in-place paths. This is the first move in
  several cycles to lower median and tail in the same step rather than
  trading between them.
- The two losing variants repeated cycle 0006's tail-redistribution
  pattern: any in-place reorder or segment promotion pays a rare
  O(position) move or added bookkeeping that widens put/get p99 for no
  median return.
- Metadata discipline is now visibly stretched. Every variant held the
  scored `Entry` at 24 bytes, but the winning index — and the losing `hot`
  segment — both grew the unscored `S3Fifo` container. The hash index in
  particular stores every resident key twice (key-queue + entries map), a
  real memory cost the scored summary does not capture.

## Open questions

- Is the hash index a true free win, or does its unscored key-duplication
  (each resident key stored twice) become the binding constraint once
  container memory is scored?
- With the resident lookup now O(1), is any scan left on the hot path to
  shorten — is workload-a now at a joint hit-rate and median-latency floor?
- Could an inline fast hasher's rare resize spike resurface as a put-p99
  regression under different inflow, even though it didn't here?

## Continuation

The lesson is decisive: stop trying to make the *linear* resident scan
shallow — both the in-place front-load (H02) and the segmented hot queue
(H03) missed the median and only widened the tail, because get-p50 is a
structural scan cost, not a hot-key-scatter problem. The productive path is
to **eliminate** the walk, not reorder it: the O(1) hash index over a
key-only store is the new incumbent and the first median+tail win together.
Next cycle should pressure-test it — specifically whether its double
storage of each resident key holds up if container memory enters scoring,
and whether any hot-path scan remains worth removing. Keep treating
hit-rate as closed (held bit-identical again) and keep the scored `Entry`
at 24 bytes, but expect unscored container growth to be the next frontier
of cost.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
