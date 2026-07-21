# Variant v002 Insight

## Helped

- metadata-size: Changing `counts: Vec<Vec<u32>>` to `Vec<Vec<u16>>` halved the CountMinSketch heap from 64 KB to 32 KB, exactly as the hypothesis predicted; the sketch heap sub-value recorded the reduction even though the summary dimension (Node bytes) was unaffected.

## Hurt

- per-op-time: Both get and put p99 latencies rose despite the smaller sketch footprint, the opposite of the hypothesis's cache-fit prediction — u16 element loads with zero-extension into u32 on every `estimate()` call appear to add enough per-access overhead to more than cancel any cache-miss benefit at this sketch size.

## Takeaway

Shrinking the sketch counter type from u32 to u16 achieved its stated memory goal but did not translate into latency gains: p99 times increased for both operations, suggesting the sketch's absolute footprint is not the binding latency constraint at capacity 1024–2048, and that narrower counter loads introduce per-access overhead that offsets the smaller working-set advantage. The hit-rate was unchanged, confirming the u16 saturation ceiling (65 535) poses no quality cost for this workload's frequency distribution.
