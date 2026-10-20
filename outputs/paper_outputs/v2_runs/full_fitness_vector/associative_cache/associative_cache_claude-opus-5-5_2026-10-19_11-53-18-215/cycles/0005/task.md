# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three hypotheses started from cycle 0001's v003 and each changed something different:
- v001 replaced the split halves with plain LRU over each merged bucket.
- v002 turned the two segments into a shared SLRU (probation and protected).
- v003 shrank `Item` metadata by replacing the wall-clock timestamp with logical 32-bit fields.

All three are Winners. v001 and v002 recovered hit rate: v001 rose above baseline, and v002 rose clearly above its parent. v003 moved metadata-size for the first time and made put much faster, with hit rate flat.

## Patterns

- Cycle 0001's suspicion is confirmed: the split-half policy was what held hit rate down. Both versions that let every key use the whole merged bucket gained hit rate, and both also removed the partner-path tail. A merged bucket with one shared policy is now the right base.
- Scan resistance matters on this trace. v002's SLRU gained hit rate without adding any per-item metadata, because segment membership is stored in the existing `len0` boundary.
- Get p50 got worse in all three variants, for different reasons. v001 and v002 scan the full bucket and memmove large ranges of 40-byte items on hits and evictions. In v003 the cause is unclear (thread-local clock access or noise). Policy and layout changes keep moving cost onto the median read.
- The three changes are independent and should combine. v003's 24-byte `Item` reduces bytes moved per shift, and that is exactly the cost v001 and v002 added. Under position-ordered policies (plain LRU, SLRU), the timestamp is not read at all.
- Taking `Instant::now()` out of the hot path helped put and tail latency. It was not the source of the get median, so cycle 0001's hypothesis on that point was wrong.

## Open questions

- Does SLRU beat plain LRU on hit rate? Each was compared only against its own parent, so they have never been measured side by side.
- Is the 3/4 protected fraction close to optimal?
- Can a compact key or tag array scanned before the full items, or a ring/offset layout that avoids front-of-Vec shifts, bring back the get median without giving up the hit-rate gains?
- Under SLRU or LRU, can the timestamp field be removed entirely to shrink metadata further, or do other policy kinds still need it?

## Continuation

Use the merged bucket with a shared policy as the new base, combined with v003's 24-byte logical-tick `Item`. Run plain LRU against SLRU on that combined base to settle which policy wins on hit rate, and tune SLRU's protected fraction. Put the main effort into making the get path cheap: fingerprint or tag scans, and layouts that avoid large memmoves on hit and evict. Drop split-half designs and any wall-clock reads on the hot path.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three hypotheses started from cycle 0002's v002 (merged-bucket SLRU). Each changed one separate thing:
- v001 moved over the 24-byte logical-tick `Item`.
- v002 cut data movement: it scans from the MRU end and limits the eviction rotate to probation.
- v003 added a per-bucket ghost list that sends returning keys straight to protected.

All three are Winners. v001 and v002 got faster with hit rate unchanged, and v001 also shrank metadata. v003 lost hit rate. Its measured latency gain has no clear cause in the code and is probably noise.

## Patterns

- SLRU's latency problem was data movement, not the policy. Two separate fixes, narrower slots (v001) and shorter or MRU-first moves (v002), each made both get and put faster with no effect on hit rate. This answers cycle 0002's get-median question in favour of layout and access order.
- Under SLRU the `timestamp` and `lfu_counter` fields are never read, so they are dead weight. Shrinking them cost nothing, which confirms that cycle 0002's policy change and metadata change are independent.
- Putting the miss path's eviction inside probation (about 1/4 of the bucket) produced the biggest put-tail improvement seen in any cycle so far. Moves that touch protected are the expensive ones.
- History-aware admission that puts ghost keys directly into protected breaks scan resistance on this LIRS trace. Protected is usually full, so each ghost promotion demotes an item that earned its place with a real second hit. A ghost hit is weaker evidence than a resident second hit, and it should not be rewarded the same way.
- Latency results can move with no code cause (v003). Small per-op shifts need re-measurement before anyone gives them credit.

## Open questions

- v001's and v002's improvements were never combined. They should stack, since each makes every remaining move cheaper or shorter.
- Plain LRU vs SLRU on the same base, and the 3/4 protected fraction, are still untested. Nobody picked them up this cycle.
- Can a ghost signal help if it only affects probation (insert with an extra life, or an ARC-style adaptive split) and never protected?
- Cost still left over: the re-find after insert walks all of protected, and a probation hit still rotates across all of protected. Could ring offsets or swap-based promotion remove these?

## Continuation

Combine v001 (24-byte `Item`) with v002 (MRU-first `find`, probation-only eviction rotate) as the new base. Consider removing `timestamp`/`lfu_counter` from `Item` completely on the SLRU path. Then attack the remaining protected-wide moves: return the insert index from `pair_put` instead of re-finding, and use ring- or swap-based promotion. For hit rate, finally test LRU against SLRU and tune the protected fraction on that base. Drop ghost-to-protected admission. Any ghost history should only influence probation or an adaptive split.


### Cycle 0004

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
