# Variant v003 Insight

## Helped

(none)

## Hurt

- per-op-time: The conditional `< MAX_FREQ` guard in `get()` failed to reduce get p50 as hypothesized, and get p99 as well as both put latency percentiles moved upward — the branch adds a compare on every `get()` call, and reduced write-touch frequency on maxed-out `freq_map` entries likely cooled those cache lines faster between eviction passes, increasing the cost of `evict_main()`'s `freq_map` reads during `put()`.

## Takeaway

Skipping the `freq_map` write for saturated entries did not lower get p50 because, on a Zipfian workload, the hot entries' `freq_map` slots are already L1-resident by virtue of constant access — the unconditional write was not a bottleneck. What the guard actually did was reduce the write-touch rate on those slots, potentially allowing them to be displaced from L1 sooner; when `evict_main()` subsequently scans `freq_map` during eviction, it finds those lines colder, pushing put latency up. The branch cost is small but the cache-residency effect works against the hypothesis for any workload where hot-entry `freq_map` slots benefit from frequent re-dirtying to stay warm.
