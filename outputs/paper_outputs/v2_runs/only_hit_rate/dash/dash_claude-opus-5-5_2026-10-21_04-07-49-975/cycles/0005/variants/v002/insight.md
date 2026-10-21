# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Putting set-dueling DIP in front of v003's fixed BIP insertion lowered hit-rate. This means the extra MRU insertion it allowed cost more than it gained on this trace. Three mechanisms in the hypothesis's design could explain the drop.
  - The `idx % 32 == 1` MRU-leader stash buckets go back to cycle-0003 MRU insertion for good. In those buckets, fresh one-touch keys again push out credited ex-residents, and the second-chance scan again ages them.
  - Whenever `psel` rose above 512, the follower buckets did the same.
  - `psel` counts every full miss whose key maps to a leader bucket, not only misses that insertion position caused. So it can drift toward MRU because of misses that BIP did not cause, such as cold misses, normal-tier evictions or loop-phase churn.

## Takeaway

Making v003's BIP insertion adaptive did not recover the moderate-reuse keys that v003's insight warned about. Overall, the result suggests that on this LIRS trace, BIP's gain is not limited to loop phases: switching to MRU in some phases or buckets costs more than it saves. With only 1/32 of stash buckets in each leader group and a miss signal that does not separate insertion-caused misses, the duel is not a reliable selector for this cache. Better next steps are the gentler changes v003 suggested: tuning `BIP_PERIOD`, or using more than one probation slot per bucket. Caveats: the overlay source files could not be found from this agent (no directory listing was available), so this analysis relies on the hypothesis's described changes. It was also measured on a single trace at a single cache size.
