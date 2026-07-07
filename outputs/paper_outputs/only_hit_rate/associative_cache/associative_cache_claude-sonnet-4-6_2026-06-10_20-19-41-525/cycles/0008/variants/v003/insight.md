# Variant v003 Insight

## Helped

- hit-rate: Decaying by 2 instead of 1 converts "lukewarm" items (counter == 2) directly to cold in a single warm-lock event, modestly enlarging the cold pool available to the cold-first scan on the next eviction call and slightly reducing how often the bucket re-enters warm-lock.

## Hurt

## Takeaway

Doubling the decay magnitude from 1 to 2 produces a real but narrow hit-rate gain on workload-a, confirming that some fraction of items occupy the counter == 2 tier during warm-lock events and that collapsing that tier in one pass rather than two measurably helps. The gain's small size suggests that warm-lock events are either infrequent enough, or that the true hot set (counters 3–4) dominates eviction selection enough, that faster lukewarm-tier demotion rarely changes which item is ultimately chosen for eviction.
