# Cycle 0004 Insight

## Cohort outcome

This cycle ran the three threads cycle 0003 left open against the 90/10
SLRU winner: push the protected ratio to 95/5 (H01), layer a
weak-protection ghost on the incumbent (H02), and add a W-TinyLFU
admission window for scan resistance (H03). Only the ratio push won —
v001 (95/5) landed in the Winners, but by the thinnest margin yet. The
ghost was barely perceptible and the admission window actively
regressed.

## Patterns

- The protected-residency lever still pays at 95/5, but its return is
  now nearly spent: shrinking probationary to ~100 slots produced a far
  smaller gain than each prior enlargement. The binding constraint has
  shifted from "appetite for protected tenure" to the probationary floor
  needed to keep draining one-hit-wonders.
- Reuse-distance memory (the ghost) is directionally right but
  structurally redundant against a maximally-protected tier: with 90% of
  capacity already protected, genuinely hot keys are retained anyway, so
  the warm-but-evicted band left for the ghost to rescue is vanishingly
  thin — and it costs a ghost structure plus an O(n) scan per insert.
- A non-recency admission signal is not just inert here but harmful: the
  ~32-slot window is far smaller than this trace's reuse distances, so it
  evicts genuinely reused keys before their second touch. Scan resistance
  assumes one-hit-wonders dominate; on this LIRS trace they do not.

## Open questions

- 95/5 still pays but barely; is there any room left above it before the
  probationary segment can no longer host the eviction path, or is this
  effectively the peak?
- Both off-axis levers (ghost, admission window) have now been tested in
  isolation *and* layered on the incumbent and failed both times — is
  there any orthogonal signal in this trace left to exploit, or is the
  recency/promotion axis fully consumed?

## Continuation

Treat the protected-ratio lever as essentially exhausted: 95/5 is the
last marginal win and probably at or adjacent to the ceiling — do not
spend a future cycle on 97/3-style probes expecting real gains. Ghost
re-admission and front admission windows are now confirmed dead on this
trace (redundant or harmful); drop both permanently. The cohort has
consumed nearly all recency/promotion structure; the next cycle should
either accept the incumbent as a converged optimum or look for a
genuinely different workload signal rather than re-tuning SLRU geometry.
