# Variant v002 Insight

## Helped

- (none): hit-rate did not improve; the change regressed it.

## Hurt

- hit-rate: Routing a `HirNonResident` reuse into the weak `HirResident` tier instead of promoting it to LIR in `insert_reuse` withheld protection from blocks that had already proven low inter-reference recency, and hit-rate fell below the parent.

## Takeaway

The SLRU "send ghost hits to the probationary tier" lesson does not transplant onto LIRS: a non-resident-HIR block being reused while still in stack S *is* the canonical LIR-qualifying signal (short reuse distance), so demoting it to the weak tier and deferring promotion by one access lets genuinely low-IRR blocks get re-evicted before they can earn LIR. S-membership and stack pruning already filter out the stale re-admits SLRU worried about, so the direct LIR promotion the parent kept was the correct behavior.
