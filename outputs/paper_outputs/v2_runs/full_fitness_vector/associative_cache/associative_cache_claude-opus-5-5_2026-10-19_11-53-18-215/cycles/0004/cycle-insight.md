# Cycle 0004 Insight

## Cohort outcome

Two variants built on cycle 0003's v001 (24-byte SLRU), and one on its v003 (ghost ring):
- v001 combined the movement fixes and returned the insert index instead of re-finding.
- v003 used ghost history to choose where a new key enters probation: cold keys go to the middle, ghost hits go to MRU.
- v002 replaced ghost-to-protected admission with an ARC-style adaptive protected cap.

v001 and v003 are the Winners:
- v001 made put faster. Its get median got slower.
- v003 gained some hit rate and made the median put faster, but its put tail got worse.

v002 lost hit rate.

## Patterns

- Put-side movement fixes now clearly stack. Evicting only from probation, keeping each move short, and returning the insert index all made put faster with SLRU semantics unchanged. Both Winners used the same short-rotate eviction, and both got the put improvement from it.
- How lookups are ordered has to match the segment layout. Scanning from the MRU end makes every probation hit scan all of protected first. That hit the get median, which is the summary for per-op time. Cycle 0003 attributed get gains to "MRU-first". That credit belonged to the shorter moves, not to the scan direction.
- Ghost history gives hit rate when it only decides where a key is placed. It loses hit rate when it decides how big protected is. v003 never displaces protected items and gained hit rate. v002 drifted toward plain LRU and lost scan resistance. It had a frequent shrink signal (ghost hits), and its grow signal rounds to zero slots in small buckets. Cycle 0003's ghost-to-protected attempt also failed. The lesson from both: anything that weakens protected hurts on this trace.
- Ghost machinery costs time on the put tail. It needs a second hash for the fingerprint, scans the ring linearly on every miss, and writes to the ring on every eviction. Its per-bucket memory does not show up in the metadata-size dimension.
- The cohort again saw latency shifts that the code does not explain. v002's slowdown may partly be cycle 0003 v003's unexplained gain going away.

## Open questions

- Would a segment-aware lookup bring back the get median while keeping v001's put gains? One order is probation MRU→LRU first, then protected. The other is a plain forward scan.
- Can the ghost fingerprint come from the existing hash (no second hash), and can the ring be checked without a linear scan? Either would shrink v003's put tail.
- Is the middle of probation the best entry point for cold keys? Would a larger ghost ring catch longer reuse distances?
- Plain LRU vs SLRU, and a fixed protected fraction other than 3/4, are still untested after three cycles.

## Continuation

Merge the two Winners. Start from v001's put path (probation-only rotate, returned index). Switch `find` back to a forward scan or a segment-aware scan. Add v003's mid-probation cold insertion. Then make v003's ghost path cheaper: derive the fingerprint from the existing hash, and give the ghost-hit and `len0 == 0` branches the short-rotate eviction. Drop adaptive protected sizing and any other mechanism that shrinks or displaces protected. If any hit-rate work is left, spend it on testing fixed protected fractions.
