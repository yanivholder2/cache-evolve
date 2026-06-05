# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Widening the ghost window from prob_cap (~20% of capacity) to capacity/2 (~50%) admitted more false-positive ghost hits — keys evicted and re-arriving at reuse distances between 410 and 1024 — which displaced genuinely warm entries from the protected segment.

## Takeaway

Expanding the ghost window to 50% of capacity produced a small hit-rate regression on workload-a. The reuse events at intermediate distances (between probationary capacity and half of total capacity) are not reliable locality signals on this trace: promoting them into the protected segment introduces noise that marginally displaces stronger candidates. The dead zone between ~20% and ~50% of capacity appears to be cold traffic on this workload, not deferred-warm traffic, so the wider ghost window works against the cache rather than for it.
