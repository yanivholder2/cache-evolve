# Variant v001 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 4 gave the most-accessed main-queue items one additional second-chance pass before eviction, retaining entries whose hot phase spans just beyond the old saturation ceiling and allowing them to serve re-requests that would previously have been misses.

## Hurt

## Takeaway

Bumping the frequency ceiling by one on top of v003's ghost-precision configuration produced a small but genuine hit-rate improvement, confirming that a non-trivial slice of this workload's items do reach the prior ceiling of 3 and benefit from the extra retention credit. The gain is incremental rather than step-change, which implies that items densely clustering at freq=3 exist but are a minority — the workload is not bottlenecked on the saturation ceiling, so further ceiling increases are unlikely to yield proportional returns.
