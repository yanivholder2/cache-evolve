# Variant v001 Insight

## Helped

- hit-rate: At the 128-wide bucket size, hm_0 gained clearly. This is the case the hypothesis aimed at. With BIP, every 32nd miss goes to the MRU end, so a working set that is changing can now get past the quarter-depth slot. Under pure LIP, a new item on a small bucket got only about 32 slots of turnover to earn a hit, which was too little. Because of this cell, the overall summary edged up.

## Hurt

- hit-rate: web_0 lost a little at both bucket sizes, and hm_0 did not move at the 1024-wide size. The MRU admission is a blind counter: it fires on every 32nd miss whether or not that item will be reused. On a trace dominated by one-hit items, this regularly puts a dead item in the protected end. It then has to travel the whole bucket before eviction, which loses part of the scan resistance LIP gave. That cost is largest on 1024-wide buckets, where a dead MRU item stays the longest.
- per-op-time: Put latency went up at both p50 and p99. The hypothesis expected it to fall, since the per-put `name() == "ClassicLRU"` string comparison is gone and 1-in-32 inserts now use `push`. In practice, 31 of 32 misses still run `Vec::insert(len/4, ..)` and shift about three quarters of the bucket. So the savings are tiny next to the new per-miss counter update, the modulo branch, and a larger bucket struct (`lip` + `bip_tick`). Get is effectively unchanged, which fits the hit path being untouched. Treat small put deltas with some caution, since run-to-run noise is of a similar size.

## Takeaway

Bimodal insertion on top of quarter-depth LIP is a trade between workloads, not a free gain. It helps a trace whose working set shifts on small buckets (hm_0, 128-wide), but it gives back scan resistance on a scan-heavy trace (web_0), and it brings nothing on 1024-wide buckets. Removing the string comparison did not visibly lower put cost, because the cost is dominated by the mid-bucket memmove. Real put-latency gains need a cheaper way to insert at depth, not trimming policy dispatch. Hit-rate gains on small buckets may need a smarter MRU-admission signal (or a period that scales with bucket size) rather than a fixed every-32nd-miss counter.
