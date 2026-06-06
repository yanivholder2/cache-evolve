# Variant v003 Insight

## Helped

- hit-rate: Deeper sketch (6 rows vs. 4) reduces hash collisions, improving the consensus frequency estimates used in admission decisions, enabling more accurate hot/cold discrimination.

## Hurt

- per-op-time (get): Additional hash function evaluations across 6 rows instead of 4 increase per-lookup cost.
- per-op-time (put): More rows require more hashing during increments and frequency estimates.
- metadata-size: Sketch memory grows 50% (65536 → 98304 bytes) due to the depth increase from 4 to 6 rows.

## Takeaway

Increasing sketch depth trades latency and memory for modest accuracy: this variant achieves a +0.33% hit-rate gain by reducing collisions across more independent hash functions, but pays ~11% latency cost on gets and a 50% larger sketch. The improvement demonstrates that collision probability remains a factor on this Zipfian workload, but the gain is incremental—depth alone does not overcome other bottlenecks (e.g., width limitations or admission threshold tuning).
