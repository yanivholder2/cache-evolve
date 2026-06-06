# Cross-Cycle Insights

## Recurring wins

- Partition ratio (small/main sizing) was the dominant lever for three
  straight cycles: enlarging main off the baseline paid repeatedly as the
  ratio moved from default toward a very thin admission filter, then
  plateaued. Each step paid less than the last — a clear, monotone knee.
- Keeping the direct-to-main ghost re-admission path *intact* (ungated)
  consistently beats every attempt to gate it. A confirmed reuse signal is
  worth more than the false re-admissions an unguarded path occasionally
  lets through.
- Once sizing converged, the first orthogonal lever to pay was *ordering
  discipline* of the main queue — making eviction recency-aware
  (move-to-front) rather than position-blind FIFO. Sharpening which
  resident the eviction scan reaches pays; it does not crowd hot keys.

## Recurring failures

- Every ghost re-admission *gate* regressed — small-queue routing,
  throttling, and freshness/position gating all discarded confirmed reuse
  signal faster than the window earned it. This axis is exhausted.
- Ghost depth/sizing on its own is inert-to-harmful: a deeper ghost
  manufactures false re-admissions from ancient keys without a gate, and a
  gate (above) regresses anyway.
- Levers that add protection or memory to weak-signal keys keep losing:
  looser promotion thresholds, frequency-ceiling deepening, and
  frequency warm-starting on re-admission all crowded hotter residents or
  bought nothing. Accumulated `get()` frequency — not a single ghost hit —
  is the discriminating signal.
- Distinct-on-paper knobs keep collapsing into the partition knob: once
  main is large and the small queue thin, the promotion threshold and the
  frequency ceiling become no-ops (entries clear or evict before the cap
  binds).

## Open questions

- Is the ordering/recency axis near its floor, or does it have more to give
  via alternative reorder triggers (e.g. reorder on promotion) or
  recency-aware tail selection? Its first win was thin.
- With sizing, promotion threshold, ghost sizing, ghost re-admission, and
  frequency warm-start all probed and only ordering paying, is remaining
  headroom small enough to declare the configuration converged?
- Are there mechanisms genuinely orthogonal to main-queue retention still
  untapped, or has every lever been shown to reduce to sizing + ordering?

## Per-policy notes

- S3-FIFO (only policy evaluated): the workload carries LIRS-style recency
  structure. It tolerates a very thin admission filter without leaking
  one-hit-wonders, so main can grow large. The partition ratio absorbs the
  promotion-gate and frequency-ceiling levers, which go inert beneath a
  thin small queue. The live axes are (1) sizing — now frozen near its knee
  and (2) ordering discipline of main — recency-aware eviction. The
  ghost/re-admission machinery should be left ungated; all tuning there has
  regressed.
