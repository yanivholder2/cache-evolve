# Cycle 0010 Insight

## Cohort outcome

This cycle closed out cycle 0009's two live threads and opened a fresh
admission-side probe: combining the two orthogonal warm-population winners
(v001), deepening the ghost signal into a frequency grade (v002), and a
TinyLFU-style doorkeeper that withholds stash admission until a key's second
sighting (v003). All three came back flat or negative; the declared winner
v002 landed exactly on its parent — a tie, not a gain.

## Patterns

- The two cycle-0009 positives overlap rather than compound. The probing-tier
  breadth rescue and the ghost gate compete for the same warm-victim pool: the
  rescue's whole job is to prevent the eviction that the ghost ring needs to
  record, so grafting them starves the ghost fast-track of its inputs and the
  combination falls below the ghost winner alone. The cross-cycle "do they
  stack" question resolves, again, to overlap.
- The dropped-then-recurring population is not just thin but shallow. Grading
  the ghost ring by hit count found almost nothing to grade — nearly every
  ghost hit carries count 1 — so deeper recurrence is not where the remaining
  hit-rate lives. The graduation mechanism is sound but inert.
- Admission filtering is net-negative on this trace. The contended tail keys
  recur quickly enough that withholding them on first sighting forfeits a
  near-term hit instead of filtering a true one-hit-wonder; admit-on-first-touch
  remains correct here. The doorkeeper protected residents that were not
  actually under threat.

## Open questions

- With seam-combination (overlap), deeper ghost recurrence (empty), and
  admission filtering (regressed) all now exhausted, is there any untried
  in-trace lever left, or has single-workload search fully saturated?
- Every probe for many cycles resolves to overlap or emptiness because the
  populations are thin on this one workload — only a second trace can tell
  whether these are workload properties or general truths.

## Continuation

Stop combining the discovered seams: the ghost gate and probing rescue are
confirmed to cannibalize each other, not stack. Stop deepening ghost
recurrence (the more-than-once population is empty) and drop doorkeeper-style
admission filtering (it regresses — the tail recurs too fast to withhold).
The protected-tier/ghost winner stands unchanged and in-trace search has now
saturated across stash, ghost, probing, geometry, and admission. The
long-deferred validation against a second, less-skewed trace is no longer
optional — it is the only remaining source of real signal.
