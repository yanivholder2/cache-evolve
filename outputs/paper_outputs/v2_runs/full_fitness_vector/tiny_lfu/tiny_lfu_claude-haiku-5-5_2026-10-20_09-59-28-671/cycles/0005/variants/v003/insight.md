# Variant v003 Insight

## Helped

- hit-rate: Deriving each sketch row's index from the single precomputed hash with the public per-row seeded mixing restores the counter indices that v001's shortcut had diverged from, so the admission decisions match the baseline again.

## Hurt

- per-op-time: Computing each row's index through the seeded mixing function on every get and put adds per-row work that v001's shortcut avoided, so both the get and put paths slowed relative to the parent.

## Takeaway

Matching the public per-row indexing from one hash recovers the hit-rate defect, but it brings back the per-row mixing cost that v001 had removed, so most of v001's speed gain is lost. Under this approach, correct indices and the speed win trade off directly. The next variant should test whether the same indices can be produced with less per-row work. The overlay source could not be read in this session, so the mechanism is inferred from the hypothesis text and the fitness results, not confirmed against the code.
