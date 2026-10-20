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
