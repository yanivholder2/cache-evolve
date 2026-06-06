## Recurring wins

- Partition sizing was the first and largest lever: progressively enlarging
  main toward a very thin admission filter (~4/96) paid repeatedly until it
  flattened into a clear knee, after which it was frozen.
- The proven recency-reordering discipline (move-to-front) pays small but real
  gains wherever it is newly applied — first on main's resident queue, later on
  the thin small admission queue under the adaptive split.
- Ghosting the *stronger* main-evicted population (keys that earned promotion at
  least once) recovered a dropped reuse signal — the first new win after sizing
  converged. Prioritizing retention of those strong records over weak one-hit
  history under a fixed budget added a further marginal gain.
- Making the partition split *adaptive* — steering the eviction threshold from
  the ghost `from_main` signal, then adding momentum and a wider clamp — was the
  first mechanism in many cycles to beat static sizing instead of collapsing
  back into it.

## Recurring failures

- Protecting, memorizing, or warm-starting weak-signal keys consistently
  regresses: looser promotion gates, frequency warm-start on re-admission
  (failed three times), and crowding hotter residents all cost more than they
  saved.
- Ghost *depth* never paid — falsified over both the weak small-evicted and the
  strong main-evicted populations. Re-accesses land inside the original
  `main_size` window or not at all; the win was always ghost *source quality*,
  never window length.
- Gating the direct-to-main ghost re-admission path (small-queue routing,
  throttling, freshness/recency splits) failed on every axis tried — a ghost hit
  is itself confirmed reuse, and gating discards that signal.
- Distinct-on-paper retention knobs collapse into the partition ratio once main
  is large: frequency-ceiling depth, promotion-threshold strictness, and
  promotion *timing* (eager vs lazy) all became inert.
- Disturbing the parent's `evict_main` rotation (in-place decrement, windowed
  victim selection, tail insertion of re-admitted/promoted keys) regressed — the
  rotation is load-bearing, refreshing survivor recency.
- Removing weak `from_main: false` ghosts entirely was a severe regression: the
  weak population is the *dominant* re-admission source and must keep being
  ghosted.
- Global time-based frequency aging over-decays hot keys; the eviction-time
  rotation already does targeted decay.

## Open questions

- Is the configuration now effectively converged for this workload? Recent
  cycles yield only sliver gains or full regressions across every probed axis.
- Does the adaptive feedback-controller framing extend to any other runtime knob
  (it failed for ghost capacity), or is the partition controller's modest,
  monotone drift the cap on adaptivity headroom?
- Is there any way to relax raw ghost *capacity* — the binding constraint — without
  leaking one-hit-wonders, given depth is dead and weak records must be kept?
- Is there a mechanism not yet touched at all, distinct from sizing, ghost
  capacity/depth/source, ordering, victim selection, insertion position, and
  promotion timing?

## Per-policy notes

- All cycles operate on one S3-FIFO-style policy (thin small admission queue,
  large main resident queue, ghost history queue, FIFO with a `freq`
  second-chance counter).
- Main queue: contention-sensitive — protecting it beats feeding it. Its
  rotation-on-eviction and front-insertion of re-admitted/promoted keys are both
  load-bearing. Move-to-front recency adds a sliver over plain FIFO.
- Small queue: too thin (~4%) and fast-churning for intra-ordering to matter
  much; the strict `freq > 1` promotion gate sets the filter ceiling, not
  insertion order. Move-to-front nudges positive only under the adaptive split.
- Ghost queue: source *quality* and source *retention priority* are live levers;
  depth, gating, warm-start, and weak-record removal are all dead or harmful.
  Capacity should stay pinned near `main_size`.
- Partition split: frozen static optimum ~4/96, then improved by source-steered
  adaptive thresholding with momentum and a wider clamp.
