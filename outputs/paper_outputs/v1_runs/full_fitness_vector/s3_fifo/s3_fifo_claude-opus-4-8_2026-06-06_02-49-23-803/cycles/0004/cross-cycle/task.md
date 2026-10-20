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
