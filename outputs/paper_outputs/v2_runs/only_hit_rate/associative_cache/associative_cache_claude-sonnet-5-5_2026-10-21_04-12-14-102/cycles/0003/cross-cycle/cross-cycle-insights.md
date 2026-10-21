## Recurring wins

- Structural separation of one-touch from re-referenced entries (probationary/protected split) is the only change with a clear gain; it won in cycles 2 and 3.
- Wider-bucket geometry gave a small gain (cycle 1) and has served as the base since.
- Tuning the split size gives small but positive gains. Most of the benefit comes from having the split at all.
- Each cycle's winner built directly on the previous winner, so incremental stacking works.

## Recurring failures

- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lowered hit rate. Full move-to-back recency is hard to beat.
- Small-window frequency-aware victim selection changed nothing measurable. A tie-break among a few old entries is too weak.
- Multi-bucket placement (two-choice) fails the slot-legality invariant, even on the baseline.
- Ghost-history ideas (ghost ring with direct promotion, adaptive targets) did not beat their parent. A ghost hit is weak reuse evidence, and promoting on it displaces entries that had earned protection.
- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. Speed and size claims remain untested, and added state (counters, rings, markers) is unpriced.

## Open questions

- Fix the evaluation so per-op-time and metadata-size are reported. Until then, no trade-off can be judged and every gain is provisional.
- Where does the smaller-probation trend turn? Is the optimum near the current size?
- Does a fixed probation size match adaptive sizing, making extra state unnecessary?
- Would a stricter promotion rule (ghost hit plus a second signal, or a real probation-to-protected frequency test) rescue history-based ideas?
- Do the wider-bucket and split gains survive once scan and shift costs are measured?

## Per-policy notes

- Plain LRU (move-to-back): strong baseline at small associativity on this LIRS trace. Promotion tweaks lose to it.
- Probation/protected split (2Q-like, within a bucket): the current best family. Smaller probation share was slightly better. Gains are now incremental.
- Frequency-aware victim choice: no effect on its own. Retest only with a wider window or as a promotion test.
- Ghost-history/adaptive variants: no gain over the fixed split. Deprioritize unless promotion is made stricter.
- Two-choice or other placement schemes: blocked by the invariant unless it is deliberately relaxed.
