# Variant v001 Insight

## Helped

- per-op-time: The position-0 early-return guard eliminates the counter write and loop entry for already-hot items, cutting get p99 latency in half (200 ns → 100 ns), which the parent's unconditional loop entry could not avoid.

## Hurt

- hit-rate: Capping promotion to a single conditional swap means an item at a mid-bucket position advances only one slot per access — on this Zipfian trace it does not converge to position 0 quickly enough, leaving warm items exposed to eviction and slightly weakening the hit-rate the parent's full bubble had built up.

## Takeaway

The position-0 guard is a genuine win for tail latency on repeated hot hits, confirming the parent's identified bottleneck, but replacing the full bubble with exactly one swap over-corrects: the ordering degrades enough to lose hit-rate despite saving write cycles. The right balance is probably a bounded loop (e.g. cap at 2–3 swaps) rather than a hard single-swap limit — enough to converge items that are close to the front while still short-circuiting the expensive long-distance promotions that rarely occur on a Zipfian workload.
