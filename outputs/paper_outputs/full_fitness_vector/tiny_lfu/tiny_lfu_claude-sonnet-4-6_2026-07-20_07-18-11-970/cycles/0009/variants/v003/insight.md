# Variant v003 Insight

## Helped

_(no dimension improved)_

## Hurt

- **hit-rate**: Expanding the protected segment to 30% did not further shield the Zipfian working set — the hypothesis assumed 20% was undersized, but the hot working set was already well-covered at 409 slots, so adding 205 more protected slots brought in lukewarm items that would have been evicted anyway.
- **per-op-time**: Shrinking the probationary pool from 80% to 70% of total capacity increases eviction pressure on that segment; items cycle through faster, causing more frequent `prot_unlink`/re-link demotion events when the larger protected zone overflows, which raised both get and put latency materially.

## Takeaway

Raising `prot_capacity` from 20% to 30% neither improved hit rate nor reduced latency: on a Zipfian trace at this capacity the hot working set already fits comfortably in the 20% protected zone, so the extra protected slots hold marginal items while the reduced probationary pool accelerates eviction churn and increases per-operation overhead. The optimal protected fraction for this workload lies at or below 20%, not above it.
