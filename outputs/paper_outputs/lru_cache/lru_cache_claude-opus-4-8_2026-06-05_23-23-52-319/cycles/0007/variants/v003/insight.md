# Variant v003 Insight

## Helped

- (none): no measured dimension improved over the parent.

## Hurt

- hit-rate: Widening the non-resident HIR horizon to `2 * capacity` retained evicted-HIR metadata for reuse distances longer than `capacity` distinct keys, and `insert_reuse` promotes every such non-resident second touch straight to LIR — so the extra-caught reuses, which are long-distance and therefore high-IRR by definition, were admitted into the protected LIR set and displaced genuinely low-IRR residents, lowering hit-rate.

## Takeaway

The hypothesis had the sign backwards for workload-a: lengthening LIRS's non-resident memory does not surface *beneficial* second touches, it surfaces exactly the long-reuse-distance ones that should stay unprotected, and the unconditional non-resident-to-LIR promotion turns that extra memory into LIR pollution. Because the harm comes from admission policy rather than horizon length, widening the non-resident bound is not a free LIRS lever — the parent's capacity-bound horizon was already at or past the optimum here.
