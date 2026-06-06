# Variant v001 Insight

## Helped

- hit-rate: The PROMO_THRESHOLD gate (≥ 8) with `was_protected` bypass marginally reduced protected-segment churn by keeping low-frequency single-access items in probationary rather than immediately displacing demoted-but-hot items, yielding a hair of improvement when layered on top of the k=4 victim scan inherited from v003.

## Hurt

## Takeaway

Gating promotion with PROMO_THRESHOLD and `was_protected` coexists with the k=4 victim scan without interference — the two mechanisms do not fight each other — but they also do not compound on this trace. The vanishingly small delta over v003 suggests that on this Zipfian workload, most probationary items either accumulate enough accesses to clear the threshold quickly, or are frequent demotees that the `was_protected` bypass immediately re-promotes, collapsing the gate's effect back toward unconditional promotion. The open hypothesis from cycle 0004 is answered: the mechanisms are compatible but additive gains require a workload where a meaningful fraction of probationary accesses involve below-threshold, never-protected items.
