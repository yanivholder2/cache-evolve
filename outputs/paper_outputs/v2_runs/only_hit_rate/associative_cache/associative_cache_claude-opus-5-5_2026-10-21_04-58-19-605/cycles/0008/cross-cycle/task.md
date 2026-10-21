# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline. Each one targeted a different axis: where new keys are inserted (v001, midpoint insertion), the cost of a hit (v002, an in-place CLOCK ring), and the size of each item record (v003, logical u32 stamps). The only measured dimension was hit-rate. v001 won on that, v002 also beat the baseline, and v003 kept the baseline's hit rate exactly, as it predicted.

## Patterns

- Both variants that improved hit-rate share the same mechanism. A new key starts with low protection: it enters at the midpoint in v001, and with its reference bit cleared in v002. It earns MRU or second-chance protection only when it gets a second hit. On workload-a, filtering out keys that are used only once matters more than keeping exact recency order.
- v002's hit-rate gain was a side effect. Its stated goal was lower latency, and it still beat LRU's hit rate. That means exact LRU ordering inside a bucket is not needed to get the gain.
- Across the whole cohort, per-op-time and metadata-size were never recorded. So the main claims of H02 (cheaper hits) and H03 (smaller items) are untested, and so is the per-op-time cost of v001's mid-Vec insert on every miss.
- v003 confirmed that the per-item wall-clock timestamp does nothing on the ClassicLRU eval path. Shrinking the item record should be safe for hit rate.

## Open questions

- Does v001's insert at the midpoint, which shifts about half the bucket on every miss, cost more per-op-time than its hit-rate gain is worth? Does v002's bit sweep on each miss offset its single-store hits?
- Does v001's insertion-position idea already fully explain v002's hit-rate gain, or does CLOCK's second-chance behaviour add something beyond it?
- Would a shared global atomic clock (v003) hurt multi-threaded throughput?

## Continuation

Next cycle, fix the evaluation gap first: make sure per-op-time and metadata-size are collected, and re-measure v001, v002, and v003 before ranking them. Then build on the low-protection-on-insert idea. Tune v001's insert depth, and try combinations: CLOCK with cleared bits on insert plus v003's smaller Item, aiming to win hit-rate, latency, and size together. Drop any further work on ordering by wall-clock time.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants refined a Cycle 0001 hit-rate winner. v001 moved insertion deeper toward the eviction end. v002 added a one-bucket ghost list that sends returning keys straight to MRU. v003 replaced CLOCK's single bit with a small saturating frequency counter (GCLOCK). v001 won. v003 improved on its CLOCK parent but still trailed the midpoint-insertion line, and v002 lost hit rate against its parent.

## Patterns

- The low-protection-on-insert filter is still the strongest lever on workload-a. Making new keys prove reuse sooner (v001) helped. Weakening the filter by giving returning keys a free promotion (v002) hurt. One recurrence at a moderate reuse distance does not predict a further near-term hit.
- Frequency information inside a bucket helps on top of plain CLOCK (v003), so a recency bit alone discards useful signal. It did not close the gap to deep insertion, though. Filtering out one-off keys still beats protecting the hot set over several sweeps.
- The evaluation gap from Cycle 0001 is still open: per-op-time and metadata-size were again not recorded. Every variant pays more on misses (more element shifts in v001, hashing plus a ghost scan in v002, longer hand sweeps in v003), and none of those costs has been measured.
- Note on lineage: v003's insight reports a contradiction with the parent insight's claim that v002 beat LRU. There is no conflict. v002 beat the baseline and trailed v001, and both statements hold.

## Open questions

- Where is the best insert depth: between 1/4 and pure LIP (index 0), or does hit rate peak and turn back down before index 0?
- Can the CLOCK/GCLOCK line catch up if new keys enter with protection below that of an existing hit, or if the counter cap is raised? Would it then match deep insertion's hit rate while keeping its cheaper hits?
- Would a more cautious ghost policy recover any gain, such as placing ghost hits partway up the bucket or using a smaller, more recent ghost ring? Or is ghost admission simply a poor fit for this trace?

## Continuation

Bracket v001's insert depth (try about 1/8 and index 0) to find the optimum, and get per-op-time and metadata-size recorded before more miss-path cost is added. One branch worth testing is a hybrid that pairs GCLOCK's in-place hits with stronger newcomer filtering, to see whether the CLOCK line can match v001 on hit rate while beating it on latency. Drop MRU-on-ghost-hit admission. Only revisit ghosts in a much more conservative form.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Two variants made the newcomer filter more aggressive: v001 moved insertion depth from 1/4 to 1/8, and v003 left the GCLOCK hand resting on the newcomer it just inserted, which gives a grace window of about one miss. v002 instead added frequency-aware victim choice within the bottom 4 slots of the 1/4-depth line. v001 won. v003 passed the previous cycle's depth winner and got close to v001, while v002's gain was barely above noise.

## Patterns

- For the third cycle running, how fast a newcomer must prove reuse is the main lever on workload-a. Both variants that strengthened the newcomer filter improved, from two different designs: shift-based LRU and in-place CLOCK. A filter as short as about 1–2 misses still did not hurt hit rate on this trace.
- Returns are shrinking along the depth axis: the step from 1/4 to 1/8 gained less than the step before it. Together with v003's near-LIP result, this suggests hit rate on workload-a is flattening near the eviction end.
- The CLOCK line's remaining gap was mostly about where the hand rests, not about frequency. Once the newcomer filter matched, in-place GCLOCK came close to deep-insertion LRU. Its no-shift design may make it the better choice once latency is counted.
- Frequency signal at the eviction end adds almost nothing when hits promote to MRU. This is the second cycle in which frequency-based or extra protection beyond the newcomer filter brought little or no hit-rate gain.
- Per-op-time and metadata-size are still not recorded, three cycles in. Every latency claim is still untested: v001's larger shifts, v002's window scan, and v003's expected O(1) miss path.

## Open questions

- Does pure LIP (index 0) help, or has v001's curve already reached its peak?
- Is the one-miss grace window over-fitted to workload-a? Traces with longer reuse distances or other cache sizes could penalise both v001 and v003.
- Once latency is measured, does v003's in-place design beat v001 overall despite a slightly lower hit rate?

## Continuation

Finish the depth bracket with pure LIP on v001, and test both v001 and v003 on at least one other trace or size to check whether the aggressive filter generalises. Make per-op-time recording a precondition. Without it, the cohort cannot be ranked between v001 and v003. Drop victim selection by frequency at the eviction end. If frequency is revisited, apply it on the promotion or insertion side, for example by setting insert depth from a key's count.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Each variant changed the newcomer filter in a different way. v001 made it stricter by moving the 1/8-depth line to pure LIP (index 0). v002 added a two-step, SLRU-style promotion on the hit side of the 1/8 line. v003 relaxed the GCLOCK line's one-miss filter for keys that recently returned, using a 4-entry per-bucket ghost ring that admits those keys with counter 1. v003 won: H03 held against its parent. v002 was flat, at noise level, and v001 regressed.

## Patterns

- The depth axis has peaked. Pure LIP lost hit rate, so the best tested depth is 1/8, which leaves about one slot of buffer behind the newcomer. On workload-a, a newcomer needs slightly more than one miss of grace. The aggressive-filter trend from cycles 0001–0003 stops here.
- The useful lever is now selective grace on top of a strict filter. Making the filter stricter for every key hurt (v001). Giving extra grace only to keys with evidence of a short reuse distance helped (v003).
- Ghost admission works only when the protection is capped. Cycle 0002 gave ghost hits a full MRU promotion and lost hit rate. v003 gives a single extra sweep pass and gains. The size of the reward matters as much as the signal that triggers it.
- Frequency on the promotion side added almost nothing (v002), because the 1/8 filter already removes most keys that are hit only once. This is the third cycle in which frequency-based protection beyond the newcomer filter did not pay off. v002's flag was also never reset after demotion, so the test was weak.
- Per-op-time and metadata-size are still not recorded after four cycles. v003 adds real cost to both: a ghost scan on every miss, and four keys per bucket. Its win is measured on hit rate alone, on one trace at one size.

## Open questions

- What ghost ring size is best: smaller, larger, or scaled to the bucket? Does the gain hold on traces with longer reuse distances?
- Does capped ghost admission carry over to the 1/8-depth LRU line? For example, a ghost hit could insert the key one step deeper than a normal newcomer. Does it help there as much as on CLOCK?
- With ghosts in place, the CLOCK line may now match or beat the 1/8 LRU line on hit rate. Once latency is measured, which line should be the main one?

## Continuation

Retire depth tuning: 1/8 is the optimum on the LRU line. Drop promotion by frequency, unless one cheap retry with the demotion flag reset properly is worth it. Build on v003: sweep the ghost ring size, try the same capped admission on the 1/8 LRU line, and validate on a second trace or cache size. Make per-op-time and metadata-size recording a hard precondition, because the CLOCK-plus-ghost lead now depends on miss-path and memory costs that have never been measured.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants extended Cycle 0004's capped ghost admission. v001 grew the GCLOCK ghost ring from 4 to 16 entries. v002 moved capped ghost admission onto the 1/8 LRU line, where ghost hits enter at index 3 instead of index 1. v003 added a per-bucket adaptive duel between strict and lenient newcomer grace on GCLOCK. v003 is the listed winner, but only because it tied its parent. It stays on the strongest line in absolute terms, yet H03 was refuted and nothing new was learned about leniency. v002 was the only variant to beat its parent, and only by a small margin. v001 regressed.

## Patterns

- The ghost signal works best when it is narrow. A small, recent ring is selective. Stretching it to cover keys with longer reuse distances diluted the counter-1 bonus, and those keys pushed out more valuable residents. Ring size behaves like the depth axis did in Cycle 0004: the peak is at or below the current setting.
- Capped ghost admission carries over between policy lines. It helped on both CLOCK (Cycle 0004) and shift-based LRU (v002). That supports the rule from earlier cycles that the size of the reward matters more than the signal that triggers it. The LRU gain is smaller, because the slot behind a 1/8 newcomer already catches most keys with short reuse distances.
- Adaptive controllers fail when they learn from their own outcome. In v003's duel, strict mode produced the negative evidence that kept the bucket in strict mode. The lenient branch almost never fired, so H03 was never really tested.
- This cycle produced no hit-rate gain on the CLOCK-plus-ghost line. Its lead now rests on Cycle 0004's result alone. Per-op-time and metadata-size are still not recorded after five cycles, and all three variants added miss-path scans or per-bucket state.

## Open questions

- Is a ring of 2–3 entries as good as 4, or better? This would also lower miss-path and memory cost.
- Would a properly built duel ever choose leniency? That needs set-dueling leader buckets, negatives counted only for newcomers admitted at counter 0, and a seed near the threshold.
- Is v002's small margin above run-to-run noise? Does it, or any CLOCK-line gain, hold on a second trace or cache size?

## Continuation

Shrink the GCLOCK ghost ring (try 2 and 3). Drop larger rings. Retry the adaptive grace duel at most once, with DIP-style leader buckets and an unbiased negative signal. Otherwise conclude that strict filtering plus capped ghost admission is the right shape. Keep ghost-on-LRU (v002) as a secondary line only. The hit-rate axis on workload-a looks close to saturated, so the next cycle should give priority to recording per-op-time and metadata-size and to validating on a second trace or size, ahead of further hit-rate tweaks.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were narrow follow-ups to Cycle 0005's continuation. v001 shrank the GCLOCK ghost ring from 4 to 2. v002 retried the leniency duel with DIP-style fixed leader buckets and a global PSEL. v003 limited ghost recording on the 1/8 LRU line to keys that were never hit. All three hypotheses were refuted and no variant beat its parent. v002 is the listed winner only because it lost the least. It adds no new mechanism worth keeping, and its own insight argues for dropping the duel.

## Patterns

- Every tweak to the ghost signal lost a little, so the current settings are already locally optimal. On ring size, 2 lost slightly and 16 (Cycle 0005) lost more, so the peak is at or near 4. On ring admission, filtering out previously-hit keys removed useful ghost hits on the LRU line. Ring length, ring admission and newcomer grace have now each been probed from both sides on workload-a.
- Newcomer leniency is closed on workload-a. Even with fixed leaders that removed the self-referential bias from Cycle 0005, the lenient one-sweep grace did worse than strict hand-on-newcomer. The duel also costs hits just by keeping lenient leaders running. Capped ghost admission already gives returning keys all the protection they need.
- Lessons about ghosts do not carry over cleanly between lines. Cycle 0005's "narrow the ring" result came from CLOCK, where long-reuse keys diluted the ring. On LRU, a hit key that has aged to index 0 has a reuse distance just past the bucket, which is exactly the population the ghost reward is meant to rescue. The two lines' ghost rings select different populations, so the same filtering rule behaves differently on each.
- All losses this cycle were small, and some may be within noise on one trace at one size. Workload-a hit rate looks saturated for the current design family.
- After six cycles, per-op-time and metadata-size are still not recorded. That is why v001's real saving (half the ghost memory, half the miss-path scan) counted for nothing. With hit rate saturated, these unmeasured dimensions are now what decides between candidates.

## Open questions

- Does a 3-entry ring match 4? This only matters once metadata-size or miss-path cost is measured.
- Does strict filtering plus a capped ghost counter hold up on a second trace or cache size, especially one with longer reuse distances?
- On the LRU line, would changing ghost-hit insert depth or ring length help where changing ring admission did not?

## Continuation

Treat strict hand-on-newcomer plus a 4-entry capped ghost ring (cycle/0004/v003) as the settled CLOCK-line shape. Drop newcomer-leniency duels, ghost-ring admission filters, and rings larger than 4. Do not spend another cycle on single-knob hit-rate tweaks on workload-a. The next cycle should get per-op-time and metadata-size recorded, or a second trace or cache size into the evaluation, and then compare the CLOCK+ghost and 1/8 LRU+ghost lines (and a 2- or 3-entry ring) on those axes. If the hit-rate search continues, the only lever left untested on the LRU line is ghost-hit insert depth.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant took a different thread from Cycle 0006's continuation. v001 swapped the CLOCK-line ghost ring's stored keys for 16-bit fingerprints, a cost-only change. v002 inserted ghost hits on the LRU line at 3/8 depth instead of 1/4. v003 added a per-bucket miss-frequency sketch as a second admission signal on CLOCK. v001 won by matching its parent's hit rate exactly. H01 held on hit rate, but its real gain (about one-eighth of the ghost memory and a cheaper miss-path compare) still can't be seen, because the evaluation doesn't measure metadata-size or per-op-time. v002 slipped very slightly and v003 regressed clearly, so H02 and H03 were both refuted.

## Patterns

- The ghost ring only needs to answer "was this key evicted recently?" A membership test is enough, so a compact fingerprint ring loses nothing. This is the first change since Cycle 0004 that lowers cost on the settled CLOCK shape without moving hit rate.
- On LRU, ghost-hit insert depth peaks at 1/4. Going deeper gave nothing back, and the curve looks flat there. That was the last single-knob lever left on the LRU line. Depth, ring size, ring admission and insert depth have now all peaked at or before their current settings.
- The admission reward must stay rare. v003 kept the reward capped but widened the signal that grants it. Its small, shared, slow-ageing sketch saturated from hash collisions and ended up granting counter 1 to most newcomers, which in effect turned off the strict one-miss filter. Cycle 0004 found that a capped reward is necessary. This cycle shows it isn't sufficient: the trigger also has to stay selective.
- This is the second cycle running with no hit-rate gain on workload-a. The design family looks saturated on that trace. Seven cycles in, metadata-size and per-op-time are still unrecorded, and v001's win depends entirely on them.

## Open questions

- Would a miss-frequency signal with controlled error help? That would mean more counters, a short window, or a threshold that scales with the window. Or is the 4-entry exact ring already the complete admission signal for workload-a?
- Does fingerprinting the ghost ring work the same way on the LRU line?
- Once costs are measured, does the fingerprint ring (perhaps shrunk to 3 entries) make CLOCK+ghost the clear main line over LRU+ghost?

## Continuation

Adopt cycle/0007/v001 (CLOCK, strict newcomer filter, 4-entry fingerprint ghost ring) as the base going forward. Retire LRU-line insert-depth tuning and treat LRU+ghost as a reference line only. Drop broad frequency-based admission signals. Retry one only if its false-positive rate is bounded by design. Stop running hit-rate-only cycles on workload-a: getting metadata-size and per-op-time recorded, or a second trace or cache size added, is now required before any candidate can be told apart.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants tested a single knob on the cycle/0007/v001 CLOCK base (strict newcomer filter, 4-entry fingerprint ghost ring). v001 raised the first-hit promotion from 1 to 2. v002 recorded ghosts only for keys never hit while resident. v003 lowered the counter ceiling from 3 to 2. All three hypotheses were refuted, and each variant lost a small amount of hit rate. v002 is the listed winner only because its loss was the smallest. Its own insight says the ghost-admission filter gives no real leverage, so it adds no mechanism worth keeping.

## Patterns

- Hit-side protection now has a bracket on both sides. A bigger first-hit step (v001) and a lower ceiling (v003) both lost hit rate, so the uniform +1 step with a ceiling of 3 is locally optimal. Too much protection shields once-touched keys, and too little evicts keys that really are reused. Every GCLOCK knob has now been probed in both directions on workload-a.
- Ghost-ring admission filters lose on both lines, but for different reasons. On LRU (Cycle 0006), excluding never-hit keys hurt. On CLOCK, excluding hit keys hurt, though only barely: with the hand resting on the newcomer, most evictions are already never-hit keys, so the filter removes only a thin slice. Aged-out residents that were hit at some point are worth recording on both lines. The unfiltered 4-entry ring is already complete.
- One reward at the same small size works for both a ghost return and a first hit. Cycle 0004 found that the admission reward must be capped. This cycle finds the same cap applies on the hit side.
- This is the third cycle in a row with no hit-rate gain on workload-a, and the third in which the continuation asked for cost metrics that never arrived. v003's shorter worst-case sweep is a real cost saving that the evaluation still can't see.

## Open questions

- Would raising the ceiling *together with* a larger first-hit step help, so that the extra reward doesn't flatten the frequency range? It's untested, but the trend in this cohort suggests a small effect at best.
- Is ceiling 2 worth taking once per-op-time is measured? Its hit-rate loss is small and its worst-case sweep is a third shorter.
- Do any of the losses in the last three cycles exceed run-to-run noise on a second trace or cache size?

## Continuation

Treat cycle/0007/v001 as final for hit rate on workload-a. The design is fully bracketed: ring size, ring admission, newcomer grace, hit-step size and counter ceiling all peak at their current settings. Drop all single-knob CLOCK hit-rate probes, including ring-admission filters and hit-reward magnitude. The next cycle should not propose another hit-rate-only tweak on workload-a. It should get per-op-time and metadata-size recorded, or add a second trace or cache size, and then re-judge ceiling 2, a 3-entry fingerprint ring and the LRU+ghost reference line on those axes.


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
