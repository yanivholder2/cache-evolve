# Variant v003 Insight

## Helped

- hit-rate: Doubling `SKETCH_WIDTH_FACTOR` from 4 to 8 halved the expected per-bucket collision rate in the count-min sketch, giving the admission gate cleaner frequency estimates and allowing it to correctly admit more hot keys that were previously blocked by over-estimated cold-key counts.

## Hurt

- per-op-time: The wider sketch (heap grew from 65 536 to 131 072 bytes at cap=1024) increased L2/L3 cache pressure on sketch reads and writes, visible as the get p99 rising from 100 ns to 200 ns even though p50 stayed flat.

## Takeaway

Widening the count-min sketch from 4× to 8× capacity does improve hit rate, confirming that collision noise was suppressing admission of genuinely hot keys — but the accuracy gain comes at a tail-latency cost because the doubled heap spills out of L2 on hot paths. The p50 being unaffected while p99 doubles suggests the cache-miss penalty is sporadic rather than systematic, so the hit-rate benefit likely outweighs the latency regression in throughput-oriented workloads, but width-scaling alone is not a free lunch.
