## Recurring wins

- Only one cycle exists so far, so nothing has repeated yet. The candidates below are from cycle 0001 alone.
- Implementation mechanics gave the only gain so far. Hashing each key once per op, storing each bucket in one contiguous allocation, and updating the policy in place made both get and put faster, most of all put's tail latency. This seems to be independent of the eviction policy, so it is the current baseline to build on.

## Recurring failures

- Any design that lets a key live outside its single hashed bucket fails the P2 slot_legality invariant. This covers two-choice placement, cuckoo displacement, and cross-bucket partner or victim areas. These designs failed in every attempt in cycle 0001. Treat this as a hard constraint, not something to tune around.
- Splitting LRU inside a bucket into home and partner halves, with demotion and promotion rules, did not raise hit rate. The bookkeeping cancelled out the extra flexibility, and hit rate came in slightly below baseline.
- The hit-rate goal has not been met in any cycle. Speed has improved and hit rate has not.

## Open questions

- Does plain single LRU over the merged, larger buckets keep the speed gain while matching or beating baseline hit rate? This test would separate the effect of the mechanics from the effect of the policy.
- Can policy changes that stay inside the bucket raise hit rate? Candidates are frequency-aware eviction, scan resistance, and admission filtering or insertion-position tuning.
- Does refreshing the timestamp on every hit (a wall-clock read) cost measurable per-op time compared with a cheap logical counter?

## Per-policy notes

- LRU (split home/partner halves): passed the gates only after merging pairs into one physical bucket. It was faster but slightly worse on hit rate, and hot halves may push out their partner's native items.
- Multi-location placement (two-choice, cuckoo): ruled out by slot_legality, whatever eviction policy sits on top.
- Frequency-aware, scan-resistant, and admission-based policies: not tried yet. These are the main untested direction for recovering hit rate.
