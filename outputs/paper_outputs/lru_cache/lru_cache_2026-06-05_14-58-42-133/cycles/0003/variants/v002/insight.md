# Variant v002 Insight

## Helped

- (none): the W-TinyLFU window did not raise hit-rate on this workload at size 2048.

## Hurt

- hit-rate: replacing the parent's "leave-the-arena-untouched" rejection gate with a window that *always* admits the newcomer means every new key now displaces a resident, and the candidate→main gate uses `>=` (admits on ties) where the parent used strict `<` rejection — a more lenient admission that lets marginal keys churn out warmer residents.
- hit-rate: at capacity 2048 the window is only `(2048/100)=20` slots, too small to give bursty keys a meaningful recency buffer before the frequency gate judges them, so the recency benefit W-TinyLFU is supposed to add never materializes while the extra churn cost is paid.

## Takeaway

For this trace the parent's plain-TinyLFU rejection gate (no eviction on a cold miss) was stronger than the Caffeine-style three-region layout: a ~1% window is too small to buffer short-reuse keys here, and the layout's always-admit-into-window plus tie-admitting candidate gate loosened admission enough to slightly lower hit-rate. The recency-window failure mode the hypothesis targeted does not dominate this workload — admission strictness matters more than a small recency buffer.
