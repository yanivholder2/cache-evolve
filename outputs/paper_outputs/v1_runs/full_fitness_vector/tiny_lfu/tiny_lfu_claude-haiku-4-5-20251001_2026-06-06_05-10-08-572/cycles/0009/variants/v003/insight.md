# Variant v003 Insight

## Helped

- **hit-rate**: Asymmetric admission gate protects cache residents from low-frequency new arrivals by requiring stricter frequency advantage (`incoming_freq > victim_freq + 1`) for insertions while always admitting updates to existing items, reducing churn from unproven newcomers displacing proven cache residents.

- **per-op-time (get)**: Higher cache hit-rate from admission policy reduces the proportion of expensive miss-handling paths relative to successful lookups, improving average get latency.

- **per-op-time (put)**: Stricter admission gate for new items reduces the rate of successful insertions competing for eviction slots, lowering put operation cost through fewer admission decisions and evictions.

## Hurt

(None)

## Takeaway

Asymmetric admission policies can improve cache efficiency by distinguishing between updates to proven residents (always admit) and risky new entries (require stronger evidence). This variant shows that a frequency-threshold offset for new items—while keeping updates frictionless—successfully balances cache continuity against churn, improving both hit-rate and operational latency without increasing metadata overhead. The asymmetry exploits the insight that existing cache members have already demonstrated utility, while new items remain statistically unproven even at marginal frequency advantages.
