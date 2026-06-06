# Variant v002 Insight

## Helped

- hit-rate: Increasing sketch depth to 6 rows provides more independent hash functions (each seeded differently), reducing collision variance in frequency estimates and allowing the admission gate to make more precise decisions about which items to retain at capacity boundaries.

## Hurt

(none observed)

## Takeaway

Increasing Count-Min sketch depth from 4 to 6 modestly improves hit-rate by strengthening hash function independence for frequency estimation. This validates that sketch parameter tuning—specifically depth over width—can improve admission precision, though the small gain suggests the admission decision is not the primary limiting factor for this workload.
