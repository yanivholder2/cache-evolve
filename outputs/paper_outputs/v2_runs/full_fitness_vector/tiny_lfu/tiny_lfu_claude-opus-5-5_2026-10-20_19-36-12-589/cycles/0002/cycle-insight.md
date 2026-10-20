# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's v003, the fast-hash version. v001 added Caffeine-style sketch aging with a 4-bit cap. v002 packed the list links into compact `u32` indices. v003 picked the victim as the coldest of a small sample of LRU-end entries. Only v002 produced a real gain, a halved per-entry `Node` size. No variant raised hit rate, and all three regressed on per-op time at least partly.

## Patterns

- Hit rate is still flat or worse. Aging (v001) and frequency-aware victim choice (v003) both tried to use the sketch more, and neither helped. Aging alone made hit rate worse: under the strict `>` admission rule with ties going to the victim, coarser counts mean more ties, so more good newcomers get rejected. Victim sampling changed nothing, because the entries near the LRU end have nearly equal estimates.
- The admission rule and the counter resolution interact. Changes to the frequency signal can't be judged on their own while the tie-break rule and the lack of a window or doorkeeper stay fixed.
- Any extra work in the hot path shows up in latency right away. Per-op data-dependent branches and bookkeeping (v001), and extra per-miss hashing (v003), each gave back latency that cycle 0001 had gained.
- Layout-only changes are the one reliable win. v002 shrank the metadata with no change in behaviour. Its latency regression has no credible cause in the code and looks like benchmark-environment noise.

## Open questions

- Is v002's per-op regression real? Benchmarking it side by side with cycle 0001's v003 on the same machine would settle this.
- Would aging help if paired with a tie-aware or randomised admission rule, or with a small admission window or doorkeeper, and with cheaper reset bookkeeping?
- Is workload-a's hit rate close to its ceiling for a TinyLFU-family design, so that efficiency is the only axis left to improve?

## Continuation

Make v002 (compact links) the new base, and first re-measure its latency against cycle 0001's v003. Drop LRU-end victim sampling. Don't re-submit aging on its own. If you revisit aging, pair it with a change to admission or tie handling and keep the bookkeeping out of the per-op hot path. Layout or efficiency changes that add no hot-path work remain the safest bets.
