## Recurring wins

- Scan-resistant policy changes (CLOCK reference bit, low-position/LIP-BIP insertion) are the only thing that has moved hit rate so far. They stop one-hit-wonders from displacing reused entries.
- Cheap, stateless or near-stateless mechanisms beat heavier ones. Gain ranked inversely with bookkeeping cost.
- Latency tracks bookkeeping cost. Removing reordering (reference bit) helped put and the get tail. Adding per-hit work hurt.

## Recurring failures

- Position-only tweaks to LRU and geometry (wider buckets, partial promotion) gave no or negative hit-rate change. Associativity and capacity do not appear to be the limit on workload-a.
- Item-movement micro-optimizations (rotate vs remove+push) did not speed up get. Item movement is not what dominates get time.
- Per-slot frequency counters give marginal hit gain and add latency on every hit and put.
- Unexplained latency gains (put in cycle 0001, one variant's latency in cycle 0002) have shown up on variants that did not touch that path. Treat them as noise or artifacts until they reproduce.

## Open questions

- Do the scan-resistant gains stack (low-position insertion plus a reference bit)?
- Where does median get time go (hashing, bucket lookup, dispatch)? No cycle has measured it, and the lookup path is the main remaining suspect.
- Do the policy gains generalize beyond workload-a, or are they specific to its scan-heavy traffic?
- Is frequency information redundant once scan resistance is in place?
- Would ghost-list or admission-filter policies add anything over the cheap mechanisms?

## Per-policy notes

- LRU baseline: insensitive to bucket width. Partial promotion hurts hit rate because a hit item that is not fully promoted becomes the next victim.
- CLOCK reference bit: wins on hit rate and makes put and the get tail cheaper by removing reordering.
- Low-position insertion (LIP/BIP-style): clearest hit-rate gain, with no extra state.
- Frequency counter (2-bit, LRU tie-break): small hit gain, but slower put and get. Lowest priority of the three.
- Metadata size has not grown in any variant so far.
