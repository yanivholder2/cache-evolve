# Variant v002 Insight

## Helped

- **per-op-time (median)**: Reducing sketch depth from 3 to 2 cuts the hashing loops by 33%, delivering ~20% median latency improvement for get operations and ~13% for put operations, confirming the direct cost-benefit of fewer sketch rows.
- **metadata-size**: Sketch heap footprint shrinks by exactly 33% (49KiB → 33KiB) from removing one depth layer, validating the fixed memory-depth relationship.

## Hurt

- **hit-rate**: Drops from 0.7354 to 0.7268 (~1.2% loss), revealing that the third sketch row provides essential collision avoidance for frequency estimation accuracy; fewer rows cause overlaps that corrupt victim-selection decisions.
- **per-op-time (tail latency for get)**: Get p99 latency degrades dramatically from 208ns to 501ns (~141% regression), indicating that increased sketch collisions induce high variance in frequency estimates, leading to poor victim choices and occasional cache misses that spike tail latency.

## Takeaway

Depth reduction from 3 to 2 achieves the predicted median latency and memory wins, but exposes a critical vulnerability: at a Zipfian workload scale, the third depth layer is not redundant—it provides collision tolerance that stabilizes frequency estimates and prevents tail-latency pathology. The hit-rate loss signals that sketch collisions directly degrade eviction quality; the get p99 spike suggests that wrong victim choices cause cascading misses. Depth-3 appears to be the practical minimum for this cache's operational scale, not a conservative safety margin. Depth-2 trades away reliability for modest per-operation gains that are negated by tail-latency blowup.
