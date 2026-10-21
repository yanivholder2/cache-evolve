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
