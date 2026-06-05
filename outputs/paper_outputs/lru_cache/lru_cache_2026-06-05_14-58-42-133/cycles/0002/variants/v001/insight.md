# Variant v001 Insight

## Helped

- (none): no measured dimension improved; the ghost directory added state and bookkeeping without producing a gain on workload-a.

## Hurt

- hit-rate: Readmitting any returning ghost key *directly into the protected segment* over-promotes keys on the strength of a single reappearance, so the `demote_one` overflow check repeatedly evicts genuinely twice-hit protected keys to make room — diluting the protected segment instead of protecting it, the opposite of the scan resistance the parent SLRU earned.

## Takeaway

On workload-a, an A1out-style ghost queue that grants protected-segment admission on one return is too generous: a returning probationary victim is often still a low-reuse key, and fast-tracking it past the second-touch promotion gate displaces hot keys and regresses hit-rate below the plain SLRU parent. The promotion gate, not the eviction memory, is what carried the parent's gain — bypassing it costs more than the recovered ghost hits return.
