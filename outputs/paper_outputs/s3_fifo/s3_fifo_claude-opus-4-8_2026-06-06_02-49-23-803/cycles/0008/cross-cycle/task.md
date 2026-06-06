# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-knob tweaks to the baseline S3-FIFO,
each aimed at lifting hit-rate on workload-a by retaining more
reuse-proven entries. Only the capacity-split change (shrinking the
small admission queue) actually raised hit-rate; the two "admit/protect
more aggressively" changes both lowered it, and the cohort's lone real
winner was the bucket-resizing variant.

## Patterns

- Two distinct strategies were tried: **looser retention** (promote on
  first reuse; re-admit ghosts at non-zero freq) and **reallocation**
  (move capacity from small to main). Both retention-loosening moves
  hurt hit-rate; the reallocation move helped.
- The baseline's "strictness" is load-bearing: the two-touch promotion
  threshold and the freq=0 ghost re-admission both act as noise filters
  that keep short-lived, weakly-reused items from overstaying in main.
  Weakening either let one-touch items crowd out durable entries.
- Giving the main queue more *room* beats giving individual entries more
  *protection*: the small queue only needs to be wide enough to screen
  one-hit-wonders, and freed slots are better spent on main residency.
- Reported per-op-time movements appear across variants that add no
  hot-path work and don't change the scanned resident-set size — these
  read as measurement variance, not real effects.

## Open questions

- How far can the small/main split be pushed before the small queue is
  too narrow to screen one-hit-wonders effectively?
- Are admission-side and retention-side knobs genuinely orthogonal, or
  does a smaller small queue change the value of the promotion
  threshold?

## Continuation

Pursue capacity-allocation knobs (small/main split, ghost sizing) — that
is where hit-rate moved. Drop the "make main stickier" family
(first-reuse promotion, non-zero ghost re-admission): on this workload
the baseline's strict filtering is protective, not over-aggressive.
Next cycle should sweep the split ratio further and consider pairing it
with the *unchanged* strict thresholds rather than loosening them.
Treat per-op-time deltas as noise unless a change touches the hot path
or resident-set size.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested three capacity/retention knobs on workload-a, branching
mostly off cycle 0001's split-ratio winner: pushing the small/main split
narrower, widening the ghost queue, and raising the frequency saturation
ceiling. The two retention-respecting moves — a narrower admission filter
and a taller freq cap — both lifted hit-rate, while widening the ghost
queue regressed it and grew the control struct.

## Patterns

- The split sweep remains the most reliable lever: continuing to move
  capacity from the admission filter into main raised hit-rate again, but
  the gain is shrinking — the filter is nearing the point where it stops
  screening one-hit-wonders.
- Capacity knobs are **not additive**: stacking a wider ghost on the
  winning split *lowered* hit-rate. Widening reuse-distance detection
  readmitted stale long-distance keys that displaced recently-reused
  entries — this trace rewards near reuse, not far reuse.
- Rewarding *demonstrated* reuse (a higher freq ceiling for already-hot
  keys) is a small, real gain and is orthogonal to admission: it preserves
  the load-bearing strict promotion/ghost thresholds while only deepening
  durability for keys that earned it. This confirms cycle 0001's lesson —
  reward proven reuse, never loosen the filter for weakly-touched items.
- Metadata-size is a live cost: adding a control field (ghost sizing) grew
  the struct for no benefit. Prefer knobs that retune existing fields over
  ones that add state.
- Per-op-time deltas again read as run-to-run noise; no change altered the
  resident scan size, and the freq-ceiling change can only lengthen the
  eviction loop, not shorten it.

## Open questions

- Where exactly does the split stop paying off — is the next step past
  capacity/16 the one that finally under-screens one-hit-wonders?
- Is the freq-ceiling gain still positive when combined with the narrower
  split, or do the two retention/capacity levers interfere like split and
  ghost did?

## Continuation

Keep sweeping the small/main split but expect diminishing returns and
watch for the crossover where the filter under-screens — find the knee,
don't assume monotonicity holds. Drop ghost enlargement: longer reuse
distance is not rewarded on this workload, and it costs metadata. The
freq-ceiling lever is worth one more probe, ideally *combined* with the
narrow split, to test whether demonstrated-reuse durability and
capacity reallocation are genuinely orthogonal. Continue treating
per-op-time as noise unless the resident scan size changes.


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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched off cycle 0007's O(1)-get hash-index incumbent
and split their fire across the two remaining live targets: the last linear
scan on the *put* path (the ghost membership walk) and the long-frozen
metadata-size dimension. Two variants eliminated the put scan by different
mechanisms; one finally moved metadata-size off its seven-cycle plateau.
All three were Winners, hit-rate held bit-identical.

## Patterns

- The ghost membership scan was the binding put cost, now confirmed by
  removal. Both put-latency variants crushed put p50/p99 by replacing the
  O(ghost_len) `iter().any` + `retain` rebuild with an O(1) probe — one via
  a dedicated ghost HashSet, one by folding ghost state into the existing
  `entries` map as value-less tombstones. This was the last linear walk on
  any hot path; the resident scan was already gone in 0007.
- The two put fixes embody opposite memory tradeoffs for the *same*
  latency win. The separate set duplicates every ghost key (a second
  key-keyed table); the folded-tombstone variant adds no table but taxes
  the get path with an enum-discriminant match and inflates the resident
  map with ghost tombstones. The scan elimination is settled; how to pay
  for it is the new contested axis.
- Metadata-size finally moved. The `key` field — dead weight ever since
  0007's hash index made the resident key recoverable from the map key —
  was deleted, collapsing the scored layout below its frozen plateau. A
  delayed dividend of the prior cycle's index, and a free win that touched
  no lookup, eviction, or promotion path.
- Container memory is now the explicitly contested frontier the prior
  cycles flagged. Every put variant grew the unscored `S3Fifo` container
  (ghost set, ghost counter, or tombstone-inflated map) even as the scored
  per-entry cost shrank.

## Open questions

- Of the two O(1)-ghost designs, which is net-better once unscored
  container memory is weighed — the duplicated-key HashSet, or the
  tombstone-laden single map that taxes get?
- With both hot-path scans now eliminated and the scored entry shrunk, is
  any latency or memory lever left, or is workload-a at a joint hit-rate /
  median-latency / scored-size floor?
- Do the tombstone designs accumulate stale order-queue / map entries that
  degrade under longer or different inflow than tested here?

## Continuation

The scan frontier is closed: get (0007) and now put (0008) are both O(1),
so stop hunting hot-path linear walks — there are none left. The two
remaining live threads are both about *cost of the same hits*, not more
hits: (1) reconcile the two ghost-O(1) designs against unscored container
memory, since they make opposite tradeoffs for an identical latency win,
and (2) keep probing the now-unfrozen metadata-size dimension for further
dead-field deletions the hash index may have made redundant. Treat
hit-rate as closed (bit-identical at the same value for many cycles
running). Expect the next binding constraint to be container/tombstone
memory, not latency.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
