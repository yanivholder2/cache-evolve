# Variant v001 Insight

## Helped

- per-op-time: Frequency decay in ghost queue reduces the number of ghosted entries through stricter filtering (only entries with freq > 0 after decrement are retained), lowering ghost-queue search overhead and accelerating put operations (p50: 577→203ns, p99: 1462→294ns).

## Hurt

- hit-rate: Aggressive frequency decay filtering shrinks the ghost queue faster than frequency preservation can compensate, reducing ghost hits and eviction of valuable items before they can be re-accessed, dropping hit-rate from 0.7164 to 0.6566.

## Takeaway

Preserving frequency information in the ghost queue backfires when combined with decay-based filtering. While the frequency decay logic successfully reduces ghost-queue overhead and improves latency, the stricter admission criteria (requiring freq > 0 after decrement) ejects more items prematurely, starving the ghost queue of candidates for re-promotion. The net result is faster put operations but worse hit-rate, suggesting that frequency preservation requires less aggressive filtering to be effective.
