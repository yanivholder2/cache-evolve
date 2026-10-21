## Recurring wins

- Changes that preserve frequency information (tie-admit duel, single-hash row derivation, index reuse) are neutral to slightly positive. They never regress.
- Only one real, if tiny, hit-rate gain in five cycles: the looser tie-admit window-to-probation duel (cycle 3). It is the current base.
- Behavior-preserving cost refactors (hash sharing, index reuse) are safe to keep as candidate cost wins. Their speed benefit is unverified.

## Recurring failures

- Anything that discards, decays, hides or dilutes frequency information lost hit-rate on workload-a. This covers aging, a larger window, a doorkeeper, 4-bit saturating counters and admission loosened past ties.
- Sketch width reduction of any size (2x or a gentle cut) lost hit-rate. The sketch is near the width where collisions matter.
- Window-size and protected-segment tuning is flat. Hill-climbing, a larger window and protected share all returned roughly nothing.
- "Winner" is often the least-bad among losers or a tie with the parent (cycles 2, 4, 5), so the selection signal is weak.
- Every cycle has fitness reporting hit-rate only. Metadata-size and per-op-time have never been measured, so cost-oriented hypotheses cannot be judged.
- Infrastructure failure (HTTP 429 on the Caffeine jar download) blocked aging in cycle 1.

## Open questions

- Is workload-a saturated, so that no structural tweak can move hit-rate? A second trace (shifting popularity or a different cache size) is needed to tell.
- Do single-hash indexing and index reuse actually cut per-op-time? Does the cached index array cost metadata?
- Would aging or a doorkeeper pay off on a trace with real popularity drift?
- Could the 4-bit sketch recover accuracy with halving and decorrelated hashes, and would its size savings justify any remaining loss?
- Is the small tie-admit win real or noise?

## Per-policy notes

- TinyLFU / W-TinyLFU (window plus segmented main, sketch-gated admission): the baseline admission already captures most of the benefit. The window/probation/protected splits are near optimum on workload-a.
- Sketch: the u8 counters with conservative update cost no accuracy. 4-bit counters without aging, and narrower rows, do cost accuracy. Hash-derivation sharing looks safe.
- Admission: tie-admit is the useful limit. Going further, such as gating on a window hit, swaps in colder entries.
- Next steps: fix the evaluation first (add metadata-size and per-op-time, add a second trace). Then verify the cost wins. Stop hit-rate-only tuning on workload-a.
