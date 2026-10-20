# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three hypotheses tried to raise hit rate by letting a key live in more than one bucket: two-choice placement (H01), cuckoo-style displacement (H02), and paired home/partner buckets (H03). Only v003 (H03) passed the gates. It won because it got faster on both get and put, especially put's tail. Its hit rate came in slightly below baseline, so the hit-rate goal of this cycle was not met.

## Patterns

- The P2 slot_legality invariant blocks any design where a key can sit outside its single hashed bucket. v001 and v002 failed on this one invariant in all three attempts. v003 got through only because it departed from its hypothesis: it merged each pair into one physical bucket, so every key still sits in its one hashed bucket.
- Within a bucket, splitting LRU into home and partner halves with demotion and promotion rules did not improve hit rate. The extra flexibility was cancelled by the bookkeeping, and hot halves pushing out their partner's native items may make it slightly worse.
- v003's speedup seems to come from how it was built, not from the pairing policy: it hashes each key once, keeps each bucket in one contiguous allocation, and updates the eviction policy in place. That speedup should carry over to other variants on its own.

## Open questions

- Is a larger, merged bucket with ordinary single LRU (no split halves) as fast as v003, and does its hit rate match or beat baseline? This would separate the speedup from the policy.
- Can hit rate be gained inside the single-bucket rule by changing the eviction policy, for example frequency-aware, scan-resistant, or with insertion-position tuning?
- Does the `Instant::now()` timestamp refresh on every hit, added by all three hypotheses, cost per-op time that a cheaper counter would avoid?

## Continuation

Drop every hypothesis that places a key outside its hashed bucket (two-choice, cuckoo, cross-bucket victim areas). They cannot pass P2 slot_legality. Keep v003's mechanics (one hash per op, contiguous bucket storage, in-place policy updates) as the new baseline. Spend the next cycle on eviction and admission policy inside the bucket to recover hit rate. Also test plain LRU over v003's merged buckets, to check whether the split-half logic is hurting hit rate.


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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three hypotheses started from cycle 0004's v003 (ghost-guided mid-probation insertion), and each changed one separate axis:
- v001 added a parallel 16-bit tag array, scanned forward before any full-key compare, and hashed once instead of twice.
- v002 raised the fixed protected fraction from 3/4 to 7/8, with a 2-slot probation floor.
- v003 cut `Item` down to `{ key, value }`.

All three are Winners. v001 improved the get median. v002 gained hit rate. v003 brought metadata-size to its floor and made get and put faster, mostly at the tails.

## Patterns

- Each axis now has a lever that works on its own: tag scanning for the get median, protected capacity for hit rate, and stride width for metadata and tails. Because each variant changed a different axis with the others held fixed, they should combine with little interference.
- The 16-byte `Item` confirms that this design's cost is mostly memmove. A narrower stride helped every path with hit rate exactly unchanged, and the biggest gains came at the tails, where the longest shifts happen. The tag array goes after the other cost, the scan width on misses.
- v002 strengthens the cross-cycle rule that giving protected more room helps on this trace. The fixed 3/4 split had not reached its limit. The gain depends on bucket geometry: the 2-slot floor cancels 7/8 at 8 slots and shrinks protected at 4 slots.
- Carry-forward is fragile. v001's metadata-size and put regressions look like a lost carry-forward of the shrunken `item.rs`, not a cost of tagging. Its put cost cannot be judged until it runs on the 16-byte `Item`.
- Latency again moved without a code cause. v002 changed only construction-time arithmetic, yet get and put both got slower. The cause is either noise or longer protected-side rotates.

## Open questions

- On a 16-byte `Item`, does keeping the parallel tag array in sync (insert, rotate, remove) cost more on put than removing the second hash saves?
- Is v002's slowdown real? A larger protected segment makes protected-hit rotates and the ghost-hit/`len0 == 0` shifts longer. Re-run it side by side with its parent.
- Has hit rate peaked at 7/8, or would 15/16 help on large buckets? Does a probation floor of 2 starve mid-probation insertion?
- Plain LRU vs SLRU is still untested after four cycles.

## Continuation

Merge all three Winners into one base: the 16-byte `{ key, value }` `Item`, tag-first forward `find` with a single hash, and a 7/8 protected cap. Then measure the put cost of tag-array upkeep on that base. Next, try cheaper protected-side moves (ring offsets or swap-based promotion), since a larger protected segment makes those moves longer. If hit-rate work remains, probe protected fractions above 7/8 and check bucket-size sensitivity. Keep dropping anything that shrinks or displaces protected.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three hypotheses started from cycle 0005's v003 (16-byte `{ key, value }` SLRU), and each changed one separate axis:
- v001 put the 7/8 protected cap, with a probation floor that scales with bucket size, onto the 16-byte item.
- v002 skipped the rotate on protected hits in the MRU quarter (lazy promotion).
- v003 doubled the ghost ring and removed one of the two extra hashes on put.

Only v001 actually moved anything: hit rate went up as predicted and metadata stayed at its floor, but get and put both got slower. v002 was neutral on every dimension. v003 was flat-to-worse on hit rate and slower on put. All three are listed as Winners, but v002 and v003 are Winners by default, not on merit.

## Patterns

- The protected cap now trades hit rate directly against latency. Cycle 0005 v002's slowdown was real: it showed up again on the narrower stride. A larger protected segment makes protected-hit rotates longer, and a 1-slot probation makes evictions and ghost re-inserts (whole-bucket shifts) more frequent. "More protected helps" still holds for hit rate, but it is no longer free.
- Lazy promotion near MRU saves nothing. Hot keys already sit at or next to the MRU slot, where a rotate moves almost nothing. The memmove cost lives in hits deep in protected and in probation promotions, and skipping those moves is exactly what would cost hit rate. Any latency fix has to make deep moves cheaper, not skip shallow ones.
- Ghost-ring length is exhausted. A ring of half a bucket already captures the reuse that placement can exploit. Longer windows add false positives, push fresher probation items toward eviction, and lengthen the linear ring scan that every miss-path put pays. Removing a single hash is too small to offset an O(ring) scan.
- Cycle 0005's tag-first `find` was never carried into this cohort. All three variants branched from the untagged parent, so the get-median lever from 0005 is still sitting unmerged.

## Open questions

- Can ring offsets or swap-based promotion make deep protected moves cheap enough to keep 7/8's hit rate without its latency cost?
- Is a cap between 3/4 and 7/8 (or a 2-slot probation at 8 slots) a better point on the hit-rate/latency curve?
- Would a non-linear ghost lookup (tiny bitset or hashed slot) shrink the miss-path put tail at the current ring size?
- What does tag-array upkeep cost on put once it runs on the 16-byte item?

## Continuation

Build the base from cycle 0006 v001 (16-byte item, 7/8 cap) plus cycle 0005 v001's tag-first single-hash `find`, carrying over both `item.rs` and the tag array explicitly. Spend latency work on cheapening deep protected and probation moves (ring offsets, swap-based promotion) and on replacing the linear ghost scan, not on skipping near-MRU rotates. Drop ghost-ring enlargement and lazy hot-zone promotion. If you probe the cap again, treat it as a hit-rate/latency trade-off, not a pure win.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three hypotheses started from cycle 0006 v001 (16-byte item, 7/8 cap), and each changed one separate axis:
- v001 finally merged cycle 0005's tag-first single-hash `find`.
- v002 limited protected-hit promotion to move-ahead-4.
- v003 raised the cap to 15/16 on buckets of 16+ slots.

v001 is the clear Winner: get and put both got faster, and hit rate and metadata did not change. v003 gained a small amount of hit rate at no latency cost. v002 kept hit rate but got slower on get and much slower on put. It is a Winner by default, not on merit.

## Patterns

- Lookup and replacement can be tuned separately. Tag-first lookup with one hash cut latency without changing any replacement decision. Put gained more than expected, because the ghost and eviction paths now reuse stored tags instead of rehashing. Keeping the tag array in sync costs much less than the key compares and hashes it removes. That answers the open question about tag upkeep.
- Long protected rotates are not what makes the 7/8 cap slow. Protected segments are at most 14–15 items of 16 bytes each, so even a full rotate is cheap. Bounding it gained nothing and added variable-length moves. Both v002 and v003 point the other way: the cost comes from the thin probation segment. A 1-slot probation means more evictions, ghost writes and ghost scans, and more whole-bucket insert/remove shifts on put.
- The protected-cap lever is nearly used up. 15/16 still adds hit rate on large buckets, but much less than the 3/4 → 7/8 step did. Its cost was low only because it reuses the existing in-place replace path. Probation now holds cold keys for only one miss.
- Approximate recency inside protected hurt indirectly. It demoted still-warm keys into probation, and that churn showed up on the put tail even though put's own code got lighter.

## Open questions

- With probation down to one slot, how much of the median put cost on a miss comes from the linear ghost scan and ghost writes? Would a hashed or bitset ghost lookup remove it?
- Can admission decide more of the hit rate now that the cap is flattening? Examples: a frequency or ghost-based filter that keeps one-hit-wonders out of probation altogether.
- Do whole-bucket insert/remove shifts in probation eviction dominate the put tail? Would a ring-offset layout for probation alone remove them?

## Continuation

Build the base from v001 (tag-first find, single hash, tags reused on ghost and evict) plus v003's 15/16 cap. Spend latency work on the miss and eviction path: non-linear ghost lookup, and cheaper probation insert/evict shifts. Leave protected-side rotates alone. Look for further hit-rate gains in admission or ghost policy, not in a larger protected share. Drop bounded or approximate promotion inside protected, and stop pushing the cap past 15/16.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three hypotheses started from cycle 0007 v001 (tag-first find, single hash, 7/8 cap), and each changed one separate axis:
- v001 merged cycle 0007 v003's 15/16 cap.
- v002 laid out the bucket hot/cold, with tags inline and ghost/policy state boxed.
- v003 put ghost-miss cold keys at probation LRU instead of mid-probation.

v001 and v003 are Winners on merit. v001 gained hit rate with no latency cost and a better put tail. v003 is the first hit-rate gain on this lineage that leaves the cap alone, and it also made put faster. v002 did not lower get latency and only moved put cost around, so it is a Winner by default.

## Patterns

- The best changes remove work. v003's LRU entry drops a rotate, and v001's smaller probation shortens the cold-insert rotate. Both improved hit rate and the put tail at once. Changes that add indirection to save memory touches (v002's cold box and inline/heap branch) cost as much as they save.
- The cap carried forward cleanly. The merge reproduced the hit-rate target exactly on top of tag-first lookup, which confirms that lookup and replacement are orthogonal. The worry about protected-rotate cost at 15/16 did not show up.
- Get latency no longer depends on struct locality. After tag-first scanning, the working set is likely cache-resident, so get now costs instructions and data movement per hit, not cache lines.
- Admission is now the hit-rate axis with room left. Where a history-less key enters decides whether it or the just-demoted protected item is evicted next. The lever only works when probation has at least 2 slots, though, and the 15/16 cap shrinks probation. v001 and v003 therefore pull against each other: at 16-slot buckets, the merged cap leaves a 1-slot probation, where LRU and middle entry are the same.

## Open questions

- Do v001 and v003 stack, or does 15/16's thinner probation (one slot at 16-slot buckets) cancel LRU insertion there? Would a probation floor of 2 at all sizes ≥16 be a better point once cold keys enter at LRU?
- With one-hit-wonders now evicted first, does the cap even need to be this large? Could a wider probation with LRU entry beat 15/16 on hit rate?
- Would a stronger admission signal, such as a frequency sketch or a ghost-hit count, beat a single ghost bit?
- The miss-path costs are still unmeasured: the linear ghost scan and whole-bucket insert/remove shifts.

## Continuation

Merge v001 and v003: 15/16 cap plus cold keys at probation LRU, on the tag-first base. Then test the probation floor jointly with LRU entry, since the cap and admission now interact through probation width. Keep pursuing admission and ghost policy for hit rate. For latency, target per-hit work and miss-path data movement (non-linear ghost lookup, cheaper probation shifts). Drop bucket-struct layout and hot/cold splitting, and do not add indirection to the put path.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Each variant tested one axis:
- v001 stacked the 15/16 cap on cycle 0008 v003's LRU cold entry, with a probation floor of 2.
- v002 added a per-bucket count-min frequency gate for admission.
- v003 swapped the miss-path whole-bucket shifts for an in-place probation rotate, plus a branch-free ghost scan and an MRU early return in `hit()`.

Only v001 moved hit rate up, by a small amount, and it paid for it in get and put tail latency. v003 kept hit rate the same but was slightly slower. v002 lost hit rate and was slower. v002 and v003 are Winners by default, not on merit.

## Patterns

- The cap and LRU cold entry stack, but only just. The floor of 2 never took effect, because 0008 v003 already kept 2+ probation slots at 16–31. So v001 is really "15/16 on buckets of 32+ slots". The cap axis is close to saturated: making probation narrower now mostly buys longer rotates and more ghost-hit re-inserts on the tail.
- An admission signal is only worth something if it can tell keys apart. The undersized, slowly aged sketch saturated, so nearly every key counted as "seen". Admission quietly fell back to MRU entry, which undid v003's one-hit-wonder protection, and the put path got heavier for nothing.
- The miss-path memmove has now been measured, and it is not a cost. Shifting 16–32 sixteen-byte items on the minority of puts that are full ghost hits is cheap. Removing the shift saved nothing visible. The fixed extras that came with it (a ghost scan with no early exit, an MRU branch that rarely fires under 15/16) made both medians worse.
- Across the cohort, every latency regression came from extra per-op work: branches, counter updates, a larger bucket footprint. None came from bytes moved. Get and put now cost instructions per op, not data movement.

## Open questions

- Would a sketch sized correctly help? That needs enough counters for the window size divided by the threshold, index bits taken from more of the tag, and a saturation check done before measuring.
- Would a ghost-hit count, or a 2-bit ghost entry, give a better admission signal than a sketch, at lower cost?
- Is v001's small hit-rate gain worth its tail cost, compared with 0008 v003's 7/8 cap at 32+?

## Continuation

Use cycle 0009 v001 as the hit-rate base, but treat the cap as finished: change neither the cap nor the probation floor any further. Drop miss-path memmove work, branch-free ghost scans and MRU short-circuits, because data movement on the miss path is not the bottleneck. For hit rate, retry admission only if the signal is checked to discriminate before it is measured (a correctly sized and aged sketch, or ghost-hit counts). For latency, cut fixed per-op instructions on the get-hit path rather than bytes moved.


### Cycle 0010

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
