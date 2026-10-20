# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Making the second-hit promotion gate conditional on stash occupancy (promote on first hit below 0.75 occupancy, require the seen-in-stash marker at or above it) left hit-rate effectively unchanged from the parent on LIRS workload-a at size 2048, so the pressure-specific gating did not add measurable value on this trace. This is only one trace and one capacity, and the overlay diff was not inspected in this pass, so the result describes the intended change rather than a verified implementation.
