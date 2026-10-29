# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): This cell gained the most, which is what the hypothesis targeted. Letting a candidate with estimate ≥ 2 win ties in `admit()` brings back the newcomer edge that v003's dedup removed. Re-referenced, recency-driven web_0 keys in the 1310-entry window now get into probation instead of losing to equally scored residents. This cell recovered most of v003's loss with the aging clock still slow, so the strict `>` tie-break caused a large share of that regression.
- hit-rate (hm_0 @ 131072): A smaller gain from the same mechanism. Large caches have long probation queues full of keys whose estimates sit at the same low values, so ties come up often there. Breaking them toward the re-seen candidate works as a mild recency bonus.
- hit-rate (web_0 @ 16384, hm_0 @ 16384): Both small-size cells gained slightly rather than giving back v003's gain. The `cand_freq >= TIE_WIN_MIN_FREQ` guard keeps estimate-1 one-hit wonders losing ties, so the change does not let in the cold keys that v003's dedup was meant to keep out.
- hit-rate (summary): All four cells moved up, so the summary goes past v003. The tie-break added gains on top of the dedup without trading anything away.
- per-op-time (get/put p50 and p99): Every percentile is faster, and the p99 tails are far shorter. However, the diff leaves `get` and `put` byte-for-byte identical to v003. The only change is the branchless `admit()` predicate in `evict_one`, which runs only on full-cache inserts and adds no loads. This change cannot explain a speedup this large. The likely explanation is that v003's measured slowdown was benchmark noise or a layout artifact, not a cost of the dedup logic. That would confirm one of the explanations v003's insight offered.

## Hurt

## Takeaway

On top of v003's miss-dedup, a guarded tie-break (the candidate wins equal estimates only if it has been seen at least twice) makes up for the lost double count. It acts as an explicit, bounded recency bonus, and it does so without bringing back admission of one-hit wonders. This separates the two effects v003 could not distinguish: the strict-`>` tie-break, not just the slower aging clock, caused much of the large-size web_0 loss. v002 is now a better base than v003. Since the `get`/`put` code did not change, v003's latency regression should be treated as a measurement problem, not as a cost of dedup.
