# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Replacing the recency-ordered probationary floor with a strict-FIFO `A1in` admission queue plus an `A1out` key-only ghost, and promoting ghost-readmitted keys *directly* into the protected tier, evicted warmer protected incumbents via `enforce_protected_budget` faster than the rare full-`A1in`-lifetime reuse band could refill them — dropping below the incumbent's 0.7313 and refuting H03.

## Takeaway

Grafting 2Q's admission front onto the converged 95/5 SLRU subtracts hits rather than adding them: because single-hit promotion already pulls every re-touched key out of the floor, swapping the floor from LRU to FIFO changes nothing for un-promoted keys, so the only live new mechanism is `A1out`'s cross-lifetime readmission — and promoting those keys straight to protected pollutes the tier with weakly-proven keys while the readmission band stays too thin to pay for the protected incumbents it displaces. This re-confirms, from the opposite (2Q) lineage, what the parent's reprieve and the ghost arms found: on workload-a the binding constraint is protected-tier *size*, and the warm-but-evicted band is too sparse to monetize.
