# Variant v001 Insight

## Helped

_(no dimension improved)_

## Hurt

- **hit-rate**: Periodic half-decay degraded the frequency signal for genuinely hot resident items just as much as for stale cold keys, making admission comparisons against LRU victims noisier rather than more accurate on the LIRS trace.
- **per-op-time**: Adding the `ops_since_decay` branch to every `get` and `put`, plus the periodic full-table sweep of the sketch (64–128 KB of counter data), roughly doubled median latency and worsened tail latency — the decay scan is large enough to pressure L2/L3 cache and slow subsequent accesses.

## Takeaway

Blanket half-decay at `capacity × 10` intervals does not selectively age cold keys: it proportionally shrinks every counter, including those of items the cache is actively serving. On a workload (LIRS) where the hot working set turns over gradually, losing half the accumulated frequency history every ~20 000 ops erodes the sketch's ability to distinguish hot residents from warm newcomers, pushing hit rate down rather than up. The per-op latency cost of the decay sweep is also non-trivial — the infrequent but large memory scan causes cache-hierarchy pressure that raises the median, not just the tail.
