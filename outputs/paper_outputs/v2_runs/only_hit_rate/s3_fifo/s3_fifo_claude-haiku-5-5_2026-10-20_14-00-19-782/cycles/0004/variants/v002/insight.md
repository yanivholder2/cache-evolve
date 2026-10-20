# Variant v002 Insight

## Helped

- None measured. The hypothesis expected ghost-hit re-admission through the small queue to recover hit-rate lost in cycle 0003, but the hit-rate dimension moved the wrong way relative to the parent.

## Hurt

- hit-rate: Routing ghost hits into the small queue instead of directly into main means re-admitted keys must earn promotion before they count as main-queue residents, which evidently cost hits on workload-a at size 2048 rather than recovering them.

## Takeaway

Sending ghost-hit re-admissions through the small queue lowered hit-rate on workload-a, so the recovery the hypothesis predicted did not happen for this trace and size; the likely cause is that the two-hit promotion bar now delays keys that were recently evicted and still hot. Caveat: the fitness vector reports only hit-rate, so metadata-size and per-op-time were not measured, and I could not read the overlay source to confirm the ghost-hit branch edit actually landed in the built code, as H02 required before attributing the result.
