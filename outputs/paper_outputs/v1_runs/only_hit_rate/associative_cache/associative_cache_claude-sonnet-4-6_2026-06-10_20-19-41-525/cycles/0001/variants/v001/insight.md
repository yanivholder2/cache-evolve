# Variant v001 Insight

## Helped

- hit-rate: Widening each bucket from 16 to 64 slots gives the per-set LRU a 4× larger eviction pool, reducing the frequency with which still-relevant items are forced out on the LIRS workload-a trace.

## Hurt

## Takeaway

Repartitioning total capacity into fewer, wider buckets (32×64 instead of 128×16) produces a small but real hit-rate gain on the LIRS workload-a trace, confirming that the baseline's 16-slot buckets were narrow enough to cause premature LRU evictions of live items. The implementation also added a capacity-conservation guard that falls back to the identity split when integer division would lose capacity, making the repartition safe across arbitrary settings without changing its effect at the standard 128×16 configuration.
