# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Initialising ghost-promoted keys at `freq=1` extends their residency in `evict_main()` by one second-chance pass, but on workload-a those keys are not re-accessed frequently enough to justify the deferral — they displace entries that would have generated hits.
- per-op-time: The `freq=1` start means `evict_main()` must decrement and re-queue each ghost-promoted entry at least once before it becomes evictable (`freq=0`), deepening the eviction scan and raising put-path latency across all percentiles.

## Takeaway

Ghost hits on workload-a do not behave like imminent repeated accesses: starting re-promoted entries at `freq=1` protects keys that go on to be idle, evicting more useful residents in their place and lowering hit-rate. The latency increase follows directly — every extra second-chance cycle in `evict_main()` adds cost to the put path. The frequency initialisation for ghost hits should remain at 0, or this idea needs validation on a workload where ghost hits reliably predict near-term reuse.
