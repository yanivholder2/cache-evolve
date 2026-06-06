# Cross-Cycle Insights

## Recurring wins

- Main-queue **sizing** was the first and largest lever: enlarging main via the
  small/main partition ratio paid repeatedly (10/90 → ~6/94 → ~4/96) until it
  flattened. It is the dominant knob and absorbs several distinct-on-paper
  levers (promotion threshold, frequency ceiling, promotion timing all became
  inert once main was large and the small queue thin).
- Sharpening **which residents the eviction scan reaches** pays where adding
  protection does not. Making main recency-aware (move-to-front) gave the first
  post-sizing gain.
- Ghost **source quality** is a live lever: ghosting the *main-evicted*
  population (keys that earned promotion at least once) recovered signal the
  baseline dropped, and prioritizing retention of those strong records over
  weak one-hit-wonder records paid again. The discriminating axis is the
  *quality/source* of the ghosted key, not the window length.
- Gains have been monotonically shrinking: sizing gave the big wins, then
  ordering and ghost-source levers paid only slivers.

## Recurring failures

- **Feeding/crowding the main queue** with weakly-evidenced keys always
  regresses: looser promotion gates, direct long-gap re-admission, and warm-
  starting re-admitted frequency all displaced hotter residents.
- **Ghost depth** is falsified over both weak and strong populations — cooled
  keys re-access inside the original window or not at all; enlarging history
  never converts. The binding constraint is raw ghost capacity, not window
  length.
- **Frequency warm-start on re-admission** regressed three separate times;
  a one-pass survival credit only helps if the key is re-hit inside that pass,
  and too few are.
- **Gating the ghost re-admission path** (small-queue routing, throttling,
  freshness/position) consistently regressed — a ghost hit is itself reuse
  evidence and demoting it discards a confirmed signal.
- **Global/time-based frequency aging** over-decays; the eviction-time
  rotation already ages the cold tail precisely.
- **Refining victim selection / insertion position** (in-place decrement,
  windowed lowest-freq victim, tail insertion of promoted/re-admitted keys)
  produced a fully negative cohort — the parent's rotation-on-eviction and
  front-insertion of strong keys are load-bearing.

## Open questions

- Is there any way to relax the ghost **capacity** constraint without leaking
  one-hit-wonders, now that depth is dead and source-order gains are bounded?
- Is the configuration effectively **converged**? Sizing, ordering, and ghost
  axes are each exhausted after yielding diminishing returns.
- Is there a lever on a **mechanism not yet touched** at all, distinct from
  sizing, ordering, eviction selection, and ghost handling?

## Per-policy notes

- **S3-FIFO (this workload, LIRS-style trace):** the baseline's stricter
  admission filtering is load-bearing, not an oversight. The trace carries
  recency structure the partition knob never reached (hence the move-to-front
  win). The `evict_main` rotation (pop_back + push_front of `freq>0` survivors)
  does targeted decay and recency refresh and must not be disturbed. Strong
  signals (multi-hit promotion, ghost re-admission) earn front protection;
  weak signals must not be memorized or protected. Distinct-on-paper knobs
  repeatedly collapse into the sizing knob once main is large and small is thin.
