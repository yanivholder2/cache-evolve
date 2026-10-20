# Variant v001 Insight

## Helped

- hit-rate: Doubling `SKETCH_DEPTH` from 4 to 8 added four more independently-seeded rows to min away collisions, nudging hit-rate up by a single hit — confirming depth attacks overestimation in the right direction but with almost no remaining headroom.

## Hurt

- (none observed): hit-rate is the only measured dimension and it did not regress; the extra rows can only lower estimates, so the change is monotone-safe.

## Takeaway

With width already at 64 cells per key, the count-min sketch separates this trace's distinct keys well enough that taking the min over 8 rows instead of 4 catches essentially no additional false-hot admissions — the lone extra hit shows the depth axis is, like width, effectively saturated. This is the clean confirmation the rationale anticipated: estimator fidelity is exhausted on workload-a, and further hit-rate must come from a policy mechanic, not a sharper frequency estimate.
