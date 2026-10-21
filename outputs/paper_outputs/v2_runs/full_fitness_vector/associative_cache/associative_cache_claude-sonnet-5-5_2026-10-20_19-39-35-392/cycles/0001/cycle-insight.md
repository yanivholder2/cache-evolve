# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes to the baseline were tried: wider buckets (geometry), partial-promotion LRU (policy), and a rotate-based move-to-back (micro-optimization). None produced a Winner. Hit rate stayed flat or dropped, and get latency stayed flat or got worse.

## Patterns

- Hit rate looks insensitive to associativity: 4x wider buckets gave only a marginal gain, so conflict misses are probably not the bottleneck on workload-a.
- Weakening recency (partial promotion) cost hit rate. In small buckets a not-fully-promoted hit item becomes the next victim.
- The hit path is not dominated by item movement. Replacing remove+push with rotate gave no get speedup, and the p50/p99 shifts look like noise.
- Put latency moved favorably in all three variants, including the two that never touched the put path. This suggests a measurement artifact or shared-path effect, so don't treat it as a real win.
- Position-based tweaks to LRU and geometry are low-yield. Metadata size never changed because no variant added state.

## Open questions

- Is capacity or the eviction policy itself the real hit-rate limit? A policy using frequency or a reference bit is untested.
- Why did wide buckets make puts cheaper? The cause is unexplained.
- Where does get time actually go (hashing, lookup, dynamic dispatch)? No variant measured it.

## Continuation

Drop geometry sweeps and rotate/memmove micro-optimizations; both are known dead ends here. Next cycle should try policies that carry a small extra signal (frequency counter, reference bit, or a ghost/admission filter) for hit rate. For get latency, target hashing, bucket lookup or dispatch overhead rather than the item move. Discount put-latency gains until they reproduce.
