# Variant v002 Insight

## Hurt

- **hit-rate**: Access-count-based filtering gates delay promotion of genuinely useful items to hot tier—requiring two re-accesses before cold→warm→hot transition increases latency for items that would benefit from immediate hot-tier residence, and smaller hot tier capacity (6% vs. 12.5%) compounds the bottleneck by reducing space for promoted items.

## Helped

(None measured in this evaluation)

## Takeaway

The three-tier stash with access-count filtering suppresses immediate promotion in an attempt to exclude one-time stash hits from hot tier, but the workload does not suffer from hot-tier pollution—instead, it benefits from eager promotion. Requiring items to accumulate multiple accesses before advancing to hot tier raises the latency cost of secondary lookups and reduces effective hot-tier throughput relative to a simpler two-tier promotion-on-hit model. The filtering approach trades hit-rate for reduced noise, which is the wrong trade-off for this workload.
