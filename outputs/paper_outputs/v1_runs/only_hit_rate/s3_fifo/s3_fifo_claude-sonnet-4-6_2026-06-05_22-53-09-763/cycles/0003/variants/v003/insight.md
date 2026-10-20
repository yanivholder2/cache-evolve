# Variant v003 Insight

## Helped

_(no dimension improved)_

## Hurt

- hit-rate: Reducing MAX_FREQ from 3 to 2 caused hot main-queue items to exhaust their second-chance credits after only two eviction passes instead of three, so items that would have survived to serve a third-pass hit were evicted prematurely.

## Takeaway

The workload-a LIRS trace has a hot set with enough temporal stickiness that a three-pass retention window is not over-generous — the third credit is actually exercised on items that genuinely get re-accessed. Tightening MAX_FREQ to 2 cuts off that final reprieve too early, evicting live items and lowering hit rate. This establishes 3 as a lower bound for MAX_FREQ on this trace; exploration should look upward (≥ 3) or leave the knob unchanged and probe other axes.
