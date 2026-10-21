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
