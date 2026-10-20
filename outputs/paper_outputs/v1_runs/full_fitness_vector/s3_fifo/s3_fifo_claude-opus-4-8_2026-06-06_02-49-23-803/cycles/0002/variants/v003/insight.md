# Variant v003 Insight

## Helped

- hit-rate: Lifting the saturation ceiling lets main entries with many real `get` hits bank more second chances, so the highest-reuse keys survive a few extra `evict_main` decrement passes — a small but real retention gain on top of the baseline.

## Hurt

- per-op-time: A taller `freq` ceiling lengthens the `evict_main` second-chance loop for hot keys (more decrement passes before discard), so put-path eviction work can only grow, not shrink, under this change. The measured drop in get/put latency is not attributable to a one-constant ceiling raise and reads as run-to-run noise rather than a benefit of the change.

## Takeaway

Raising the frequency cap rewards *demonstrated* reuse without touching the strict `freq > 1` promotion or `freq = 0` ghost re-admission, and it nudges hit-rate up only marginally — confirming the load-bearing levers are the promotion/ghost thresholds, not the saturation ceiling. The deeper second-chance counter buys durability for already-hot keys but does not meaningfully reshape what gets admitted.
