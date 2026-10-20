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
