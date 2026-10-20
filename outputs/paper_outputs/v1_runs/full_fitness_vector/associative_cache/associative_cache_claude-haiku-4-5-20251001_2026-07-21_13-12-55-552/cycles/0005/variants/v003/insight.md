# Variant v003 Insight

## Helped

(none)

## Hurt

- **hit-rate**: Reducing bucket count from 32 to 16 increases hash collision density and worsens key distribution, causing hit-rate to decline slightly as fewer buckets cannot accommodate working-set diversity as effectively.
- **per-op-time**: Scanning 128 items per bucket overwhelms the early-exit optimization and cache-line-alignment benefits—the linear scan cost dominates for put operations (particularly eviction), pushing put latencies 3× higher, and even get p99 latencies double as branch prediction and prefetch become less effective with such large sequential searches.

## Takeaway

Pushing ultra-wide geometry from 32×64 to 16×128 reveals the micro-optimization strategy's scalability limit. While early-exit and cache-line alignment successfully controlled latency for 32×64, the compounding cost of scanning 128-item buckets—especially during insertion and eviction—overwhelms those benefits. The tighter bucket distribution also hurts hit-rate, showing that geometric extremism without proportional algorithmic advancement does not pay off.
