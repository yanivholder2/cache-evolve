# Cycle 0010 Insight

## Cohort outcome

All three hypotheses started from cycle 0009 v001 (15/16 cap, LRU cold entry, tag-first find), and each changed one separate axis:
- v001 trimmed fixed instructions on the hit path: one shared rotate for both hit kinds, branch-free `len0` bookkeeping, and no hot-path `debug!`.
- v002 replaced the linear ghost ring with a direct-mapped ghost table.
- v003 promoted probation hits to the midpoint of protected instead of protected MRU.

v001 and v003 are Winners on merit. Both cut get and put tail latency at zero hit-rate cost, though neither moved the get median. v002 lost a little hit rate and gained no latency, so it is a Winner by default.

## Patterns

- This cycle's gains were all in tail latency. Both latency wins cut the tail and left the get median flat, from two different sources: v001 removed mispredicting branches, and v003 halved the longest promotion memmove. The get median now looks set by the tag scan plus the base rotate, not by control flow or by the occasional long rotate.
- This partly revises cycle 0009's claim that "bytes moved don't matter". Bytes moved on the miss path still don't matter. On the hit path, though, the long rotate to protected MRU under 15/16 was a real tail cost, and shortening it got back most of the tail the cap had added.
- Reordering inside protected is a weak lever for hit rate. Midpoint (S3LRU-style) promotion barely changed which keys survive, because keys with exactly two hits were not crowding out hot keys on this trace. Together with the saturated cap, this means the protected segment's size and order are both used up as hit-rate axes.
- The ghost lookup question is closed. An O(1) table saved nothing over a cheap scan of a few slots. Collision-based aging also blurs the ring's exact FIFO recency window and costs hit rate. Ghost aging precision matters more than ghost lookup speed.
- v001 and v003 both rewrite `hit()` and do not merge trivially. v001's single shared rotate assumes both hit kinds rotate to the end, while v003 sends probation hits to a midpoint target. A merge needs one fast path with a per-kind rotate end and the branch-free `len0` update.

## Open questions

- Does the merged v001 + v003 fast path keep both tail gains, or does the per-kind rotate end bring back the branch that v001 removed?
- Can `find_tagged` itself get cheaper, for example with a SIMD tag compare or a narrower tag scan, now that it likely sets the get median?
- Where is hit-rate headroom left once the cap, the probation floor and protected ordering are all exhausted? Ghost-hit counts or 2-bit ghost entries as an admission signal are still untested.

## Continuation

Merge v001's branch-free hit path and dropped logging with v003's midpoint promotion, and keep them as the latency base. Check that the merge stays bit-identical to v003's replacement decisions. For the get median, target `find_tagged` and the base rotate, not more control-flow trimming. Drop all ghost-lookup restructuring and any further reordering inside protected. For hit rate, the only lever not yet ruled out is a discriminating admission signal (ghost-hit counts or 2-bit ghosts), and it should be checked for discrimination before it is measured.
