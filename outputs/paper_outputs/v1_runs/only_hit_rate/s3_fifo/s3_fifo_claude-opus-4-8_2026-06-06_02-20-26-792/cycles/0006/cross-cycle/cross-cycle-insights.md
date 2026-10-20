## Recurring wins

- Enlarging the main partition was the dominant early lever: shrinking the
  small/admission queue (10/90 → ~6/94 → ~4/96) paid repeatedly until it
  plateaued. The workload tolerates a very thin admission filter without
  leaking one-hit-wonders.
- The discriminating signal is *accumulated reuse quality*, not membership
  alone. The two levers that paid after sizing converged both sharpened on
  quality: making main recency-aware (move-to-front), and ghosting the
  *main-evicted* population (keys that earned promotion ≥ once) rather than
  weak small-evicted keys.
- The parent's `evict_main` rotation (pop_back + push_front of surviving
  `freq > 0` entries) is load-bearing across cycles — it refreshes survivor
  recency and applies targeted cold-tail decay. Repeatedly confirmed valuable.

## Recurring failures

- Crowding/protecting weak-signal keys consistently regresses: looser
  promotion gates, deeper ghost history, frequency warm-starting on
  re-admission, and gating/throttling ghost re-admission all hurt or were
  inert. A lone small-evicted ghost hit is weak evidence and should not buy
  protection.
- Distinct-on-paper retention knobs collapse into the partition knob once main
  is large: frequency-ceiling deepening, stricter small-to-main promotion
  threshold, and eager-vs-lazy promotion timing all went inert because the
  thin admission FIFO already fixes the resident set.
- Eviction/ordering refinements that removed the rotation regressed: in-place
  second-chance, windowed lowest-`freq` victim selection, and tail-insertion
  of re-admitted/promoted entries. Sharpening victim choice or demoting strong
  keys to the tail costs more than it saves.
- Blunt global mechanisms (periodic uniform frequency aging) over-decay hot
  residents; targeted eviction-time decay already does this precisely.

## Open questions

- Now that the *main-evicted* (stronger) population is proven worth ghosting,
  does ghost *depth* — inert over weak small-evicted keys — become live over
  this stronger population? Worth re-probing depth paired with source quality.
- Is the ghost queue worth splitting/weighting by source (main-evicted vs
  small-evicted), or does a single shared window suffice?
- Has the ordering axis any remaining headroom, or did move-to-front already
  capture the recency structure? Recent ordering probes were a fully negative
  cohort.
- Is overall headroom now small enough to declare the configuration converged,
  or does the reopened ghost mechanism justify further cycles?

## Per-policy notes

- S3-FIFO: optimum sits at a very thin small/main partition (~4/96), frozen.
  The main queue is contention-sensitive — protect it, don't feed it.
- S3-FIFO ghost mechanism: ghost *quality* (which population is recorded) is
  the live axis; ghost *depth* and ghost *gating* have not paid. The
  direct-to-main re-admission path for ghost hits should stay intact —
  every variant that gated, throttled, or rerouted it regressed.
- S3-FIFO main ordering: recency-aware (move-to-front) plus the
  rotation-on-eviction is the standing discipline; do not disturb either, and
  keep front-insertion of re-admitted/promoted keys.
