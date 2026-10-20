# Cycle 0010 Insight

## Cohort outcome

Two eviction-side variants qualified as Winners: a 2-bit frequency-counter victim pick on top of the aged doorkeeper (v001), and a one-bit second-chance flag on baseline with no doorkeeper (v003). The doorkeeper reset variant (v002) was rejected after three attempts produced no overlay files. This was the first cycle aimed at eviction instead of admission. Only v003 showed a gain, a small hit-rate gain with a large latency drop. v001 gained nothing.

## Patterns

- Eviction signals are a weak lever. The frequency counter (v001) did not raise hit-rate and added per-hit and per-eviction work. The one-bit flag (v003) gave only a modest hit-rate gain. Once admission is filtered, eviction choice separates good victims from bad ones very little on this trace.
- Cheaper bookkeeping did better. One bit set on hit and a single-pass scan (v003) beat a saturating counter with aging of the scanned slots (v001), which regressed both get and put.
- v003's large latency drop is out of proportion to its small hit-rate gain, on a change that adds hit-path work. It is most likely noise or layout effects, as in earlier cycles. The noise floor is still unmeasured.
- Metadata stayed flat for both, since the new state fit in spare bits.
- Winner status is again weak evidence, because flat metadata and near-flat hit-rate qualify.
- Both variant authors could not read the source, so the mechanisms are inferred from fitness vectors.
- A rejected empty overlay (v002) is an implementation failure, not evidence against incremental doorkeeper reset. That idea remains untested.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for seven cycles.
- Is v003's latency gain real? Does it persist when repeated, and when the second-chance flag is combined with the aged doorkeeper?
- Is v003's small hit-rate gain additive with the doorkeeper, or does the filter already capture it?
- Do these results hold on a second trace with scans or frequency skew, where frequency-aware eviction may matter more?

## Continuation

Drop multi-bit frequency counters and any eviction scheme that adds per-hit or per-eviction aging work. First re-run v003, its baseline parent and the doorkeeper incumbent several times to set the noise floor and confirm v003's latency gain. Then try stacking the one-bit second-chance flag on the aged doorkeeper, and test on a second trace. Retry incremental doorkeeper reset only with a clearer implementation brief, and treat it as low priority.
