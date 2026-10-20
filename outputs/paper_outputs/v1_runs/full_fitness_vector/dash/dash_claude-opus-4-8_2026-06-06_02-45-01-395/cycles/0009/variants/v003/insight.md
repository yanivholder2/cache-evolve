# Variant v003 Insight

## Helped

- hit-rate: Giving the cascade victim its own retry into its true target — instead of dropping a still-foreign displaced item straight to stash — kept a handful of conflict victims resident that the parent evicted, which is exactly the marginal residency that nudged hit-rate past the standing 0.6794 cap for the first time.

## Hurt

- per-op-time: Adding a third normal-bucket insert plus the extra `get_index(&cascade.key, …)` recomputation inside the `true_target != source_idx` branch lengthens the worst-case demotion path, and that shows up as a markedly wider put-p99 (the put side owns the cascade) and a smaller get-p99 widening (stash→target promotions on the get path also trigger the deeper cascade).

## Takeaway

The extra rescue level confirms the cascade is a real hit-rate lever, not a saturated one: spending one more put-side insert on the second-order conflict victim does buy residency the parent threw away, but the gain is barely above the cap while the put-tail cost is large and immediate. The cascade depth trades latency for hit-rate at a steeply diminishing rate — each added level buys far less hit-rate than the tail it widens, so deeper rescue past this point is unlikely to pay.
