# Variant v003 Insight

## Helped

_(none)_

## Hurt

- hit-rate: Starting ghost re-insertions at `freq: 1` extended their tenure in main, causing them to displace items already resident there that had earned their frequency through genuine accesses, producing a net increase in evictions of high-value items on this workload.

## Takeaway

Granting ghost re-admitted items a free second-chance credit does protect them from immediate eviction, but on workload-a that protection comes at the direct expense of incumbents already competing in main via the second-chance loop. The ghost re-use signal is real, but it is not strong enough to justify claiming extra residency at the cost of crowding out existing main-queue residents — the policy works better by letting re-admitted items prove their worth from `freq: 0` alongside every other entry.
