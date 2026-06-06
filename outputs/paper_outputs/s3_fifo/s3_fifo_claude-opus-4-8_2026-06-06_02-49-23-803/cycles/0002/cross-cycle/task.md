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
