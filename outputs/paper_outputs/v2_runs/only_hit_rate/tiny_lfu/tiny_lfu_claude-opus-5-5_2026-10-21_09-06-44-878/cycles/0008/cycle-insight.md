# Cycle 0008 Insight

## Cohort outcome

This cycle ran the decisive recency tests that cycle 0007 called for, each on a different parent. v001 added global periodic halving to SLRU+dedup. v002 let a ghost-hit newcomer beat the victim by a margin. v003 discounted idle residents and demoted them out of protected. All three lowered hit-rate on workload-a. v002 is the Winner only because its loss was the smallest, close to noise. This is the fourth cycle in a row with no real gain.

## Patterns

- All three ways of adding recency failed: decaying counts (v001), recency evidence in the contest (v002) and per-entry idle decay (v003). On workload-a, lifetime frequency is the asset. The keys it protects tend to have long reuse distances, and they are worth more than the stale-high keys that recency would remove.
- The sizes of the losses match how widely each change acts. Halving every count, or discounting every sampled victim, touches every eviction. v003's one-sided discount (victims discounted, newcomer not) pushed admission toward LRU and lost TinyLFU's scan resistance. v002's margin applies only to ghost keys seen at least 3 times, so its loss was tiny.
- Cycle 0007 asked whether old results against aging held only for older designs. They do not. v001 was a clean test on the current design and still lost, so it rules out the earlier confounders.
- Cycle 0007 also asked whether recency could act through protected-segment demotion. The answer is no: the protected segment already keeps keys that return just past the idle thresholds tried here, and demoting them on idle time evicts them before their next reuse.
- Two variants carry caveats. v003 bundled two mechanisms, so the cost of each is not separated. v002's author could not inspect the source, so it is unconfirmed that the change was implemented as specified.

## Open questions

- Would protected-head demotion alone (raw estimates, no discount) be neutral or still negative? This is the only part not tested on its own.
- Would a much longer halving period (well beyond 10×capacity) avoid the damage? This would be a last check, and probably converges to the unaged parent.

## Continuation

Treat workload-a's hit-rate as at its ceiling for the TinyLFU+SLRU family. Drop aging, ghost margins, idle discounts and idle-based demotion. At most, spend one variant on very long-period halving or raw-estimate demotion alone, and only to close the question. Spend remaining effort on getting per-op-time and metadata-size scored, or on other workloads, rather than on more workload-a hit-rate tweaks.
