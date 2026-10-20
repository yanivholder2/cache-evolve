# Variant v001 Insight

## Helped

- metadata-size: No change from the parent. Halving the protected-segment capacity alters only a capacity constant, so the node, sketch, and cache struct sizes stay the same.

## Hurt

- hit-rate: Slightly lower than the parent on workload-a at size 2048, and below the +0.001 gain the hypothesis predicted. Halving the protected share forces more entries out of the protected segment into probation, so frequently re-referenced items lose their protected residency sooner. That loss outweighs any benefit from the demotion path on this trace.
- per-op-time: Get and put latencies rose at both the median and the tail relative to the parent. The likely cause is that a smaller protected segment makes the demotion branch fire far more often, adding list moves to the hot access path. Only the demotion branch changed, so the cost comes from that branch being reached more often.

## Takeaway

Halving the protected share did make the demotion branch reachable, and the branch did not pay off: hit-rate dipped and per-op time got worse. This suggests that on workload-a at size 2048, protected-segment capacity matters more than protected-tail turnover, so v002's zero-movement result was not just an unreachable branch. Note: the changed source could not be inspected in this review, so the mechanism above is inferred from the hypothesis text and the fitness vectors, not confirmed in code.
