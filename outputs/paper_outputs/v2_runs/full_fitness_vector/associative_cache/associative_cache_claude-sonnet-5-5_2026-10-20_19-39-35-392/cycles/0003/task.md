# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003

## Winners

- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted scan resistance in the eviction/insertion policy: a CLOCK reference bit, a 2-bit frequency counter with LRU tie-break, and low-position insertion (LIP/BIP-style). All three became Winners with hit-rate gains, in contrast to cycle 0001's position-only tweaks. The insertion-position variant showed the clearest gain.

## Patterns

- Policy changes that keep one-hit-wonders from displacing reused entries are what moved hit rate. Capacity and associativity were not the limit.
- Gains were small to clear, and they ranked by how little the change cost. Stateless insertion position and the reference bit beat the frequency counter.
- Latency followed bookkeeping cost. The reference bit made put and the get tail cheaper by removing reordering. The frequency counter added work on every hit and put, which slowed both slightly.
- Median get stayed flat wherever the hit path got cheaper. This confirms that item movement isn't what dominates get time, so the lookup path is the remaining suspect.
- No variant grew metadata. The extra state fit into existing padding or wasn't needed.
- The v003 latency improvement is unconfirmed and may be noise or come from wide buckets. Treat it with the same caution as cycle 0001's put-latency artifact.

## Open questions

- Do the policy gains stack, for example low-position insertion plus a reference bit?
- Is the frequency signal redundant once scan resistance exists, given its marginal gain and latency cost?
- Where does median get time go (hashing, bucket lookup, dispatch)? Still unmeasured.
- Do these gains hold on workloads other than workload-a, or are they specific to its scan-heavy traffic?

## Continuation

Combine the cheap scan-resistant mechanisms (low-position insertion with a CLOCK reference bit) and drop the per-slot frequency counter, since its latency cost outweighs its small gain. Aim the next latency hypotheses at the lookup path (hashing, bucket search, dispatch) rather than eviction or item movement. Re-check any latency gain that comes without a mechanistic reason.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
