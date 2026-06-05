# Variant v003 Insight

## Helped

- hit-rate: Replacing pure LRU with SLRU plus IRR-based victim selection gave a large lift on workload-a — the trace is LIRS-generated, so IRR (gap between the two most recent accesses) is the same signal the trace generator used to decide which keys are hot, making it far more accurate than position in a recency list alone.

## Hurt

_(none — only one fitness dimension is tracked)_

## Takeaway

When the workload is known to have LIRS structure, switching the eviction signal from positional recency to inter-reference recency directly matches the cache's eviction criterion to the trace's access pattern, producing a substantial hit-rate gain. Preserving IRR timestamps across protected-to-probationary demotion avoids the signal-reset problem that would otherwise make a recently hot entry look cold as soon as it is moved, letting genuinely cold newcomers (IRR = MAX, no second access recorded) lose the eviction contest naturally without any manual counter adjustment.
