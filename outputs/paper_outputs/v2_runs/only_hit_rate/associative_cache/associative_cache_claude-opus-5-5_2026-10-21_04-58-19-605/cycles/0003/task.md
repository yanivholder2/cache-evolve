# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
