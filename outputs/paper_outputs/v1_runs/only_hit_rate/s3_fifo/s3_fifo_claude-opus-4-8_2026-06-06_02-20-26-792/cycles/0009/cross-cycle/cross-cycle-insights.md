# Cross-Cycle Insights

## Recurring wins

- Partition sizing was the dominant early lever: enlarging main (small/main from
  baseline → 10/90 → ~6/94 → ~4/96) paid step after step until the knee, where
  it froze. Most "distinct" retention knobs later collapsed back into this one.
- Ordering discipline on main pays where protection does not: explicit
  move-to-front recency beat position-blind FIFO, and the eviction-time
  rotation (pop_back + push_front with per-survivor decay) proved load-bearing —
  it refreshes survivor recency and targets cold-tail aging precisely.
- Ghost *source quality* is a live axis: ghosting the strong main-evicted
  population (keys that earned promotion) recovered a dropped signal, and
  dropping weak records before strong ones under a fixed budget paid at the
  margin.
- Adaptivity finally beat static sizing: a source-steered (`from_main`) feedback
  controller that nudges the split threshold, then momentum + a wider clamp,
  each added a small real gain — the static ratio was slightly main-starved.
- Standing principle: sharpening *which residents the eviction scan reaches*
  pays; reordering beats re-admitting or warm-starting.

## Recurring failures

- Admitting more keys into main (looser promotion, frequency-ceiling deepening,
  warm-started re-admission) crowds hotter residents — repeatedly regressive or
  inert once main is large.
- Ghost *depth* is falsified over both weak and strong populations: cooled keys
  reaccess inside the original window or never; retained-past-bound entries
  don't convert.
- Ghost re-admission gating in every form (small-queue routing, throttling,
  freshness/recency split) regressed — the direct-to-main path is correct; a
  ghost hit is itself sufficient reuse evidence.
- Frequency warm-start on re-admission regressed three separate times (weak and
  strong populations alike) — protection only pays if reuse lands inside the one
  protected pass, and too few keys do.
- Victim sharpening (in-place decrement, windowed lowest-freq selection, tail
  insertion of promoted/re-admitted keys) regressed — it discards the
  rotation's recency refresh and demotes earned-front keys.
- Global frequency aging over-decays (hot keys evicted before freq rebuilds);
  weak-ghost removal is a severe regression (weak `from_main:false` records are
  the dominant re-admission source); steering hysteresis and adaptive ghost
  capacity both regress — the drift is monotone signal not noise, and ghost
  capacity is the binding constraint that must stay pinned near `main_size`.

## Open questions

- Is the configuration now converged? Cycles 0003–0009 have produced only
  sliver gains, one fully-negative cohort, and several falsified axes.
- Adaptivity headroom appears capped by this trace's modest, near-monotone
  drift — is there any controller refinement left, or a smarter steering signal?
- With sizing, ordering, ghost depth/capacity/gating, and two runtime knobs all
  explored, is there any mechanism not yet touched that could still pay?

## Per-policy notes

- All cycles tune one S3-FIFO-style policy (thin small admission queue, large
  main, ghost history with a `from_main` source tag). The workload is a
  LIRS-style trace with recency structure and modest, monotone phase drift.
- The binding constraints are main retention (absorbed by the ~4/96 partition)
  and raw ghost capacity (must stay pinned; depth cannot productively enlarge
  it). Both weak and strong ghost populations are load-bearing — trim by source
  priority, never delete a source.
- The main queue's rotation-on-eviction and its front-insertion of
  re-admitted/promoted keys are earned, not waste; do not disturb either.
- Exhausted/falsified axes — do not revisit: static sizing, ghost depth, ghost
  warm-start, weak-ghost removal, ghost re-admission gating, eviction ordering,
  victim sharpening, insertion position, steering hysteresis, adaptive ghost
  capacity. Adaptivity (split controller) is the most recently live axis but
  looks near tuned-out.
