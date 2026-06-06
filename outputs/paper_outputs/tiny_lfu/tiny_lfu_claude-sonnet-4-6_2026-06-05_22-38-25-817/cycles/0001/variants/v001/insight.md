# Variant v001 Insight

## Helped

(none)

## Hurt

- hit-rate: periodic halving of all sketch counters every `capacity × 10` increments degraded frequency accuracy faster than the workload refreshes relative hot/cold distinctions, causing the admission gate to evict warm resident keys in favour of transient newcomers.

## Takeaway

The aging interval of `capacity × 10` increments is too short for the LIRS workload-a trace: counter magnitudes collapse before the sketch can accumulate enough evidence to reliably distinguish long-run hot keys from bursty arrivals, so the admission comparisons become unreliable and more warm residents are displaced. Sketch aging is directionally sound as a concept, but the reset cadence needs to be substantially longer (or the halving should be bounded so high-frequency keys cannot fall below a floor) before it can help rather than harm on this trace.
