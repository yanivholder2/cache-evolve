# Variant v001 Insight

## Helped

- hit-rate: No dimension improved; raising the promotion threshold to two hits produced no gain over the parent's promote-on-first-hit rule.

## Hurt

- hit-rate: Turning `lfu_counter` into a saturating 0→1→2 count and requiring the count to reach 2 before promotion leaves a once-hit key stranded in probation (at `counter == 1`) where it keeps aging in FIFO and stays an eviction candidate; keys that the parent would have protected on their first reuse now get evicted before they can land a second hit, so genuinely reusable keys are lost.

## Takeaway

For this workload one probationary hit is already sufficient proof of reuse: demanding a second hit before promotion withholds protection from rising keys long enough for FIFO probation to evict them first, costing hit-rate rather than gaining it. The extra `counter == 1` eviction sub-level the change added is exactly the trap — it is precisely where reusable keys die waiting for a promotion that never comes.
