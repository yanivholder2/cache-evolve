## Recurring wins

- Protecting hot residents at eviction works better than protecting them at insert. The full-scan, lowest-counter victim choice has produced the only consistent hit-rate gains (cycles 0002, 0004, partly 0007).
- Keeping the counter full-scan and non-decaying preserves that gain. Variants that shortened, sampled, or decayed the counter lost it (cycles 0005, 0007, 0009).
- The deferred stash-to-target promotion keeps a hit-rate gain when its first-hit bookkeeping is cheap (cycles 0003, 0004). Its evidence is limited to workload-a.
- Layout and replacement are separable. Packing or repacking records left hit rate and eviction order unchanged in every cycle that tested it (cycles 0004, 0005, 0006, 0007, 0008).

## Recurring failures

- Gating or delaying entry into the main table has lost hit rate. MRU insert displaced hot residents (cycle 0001), fullness-based insert placement cost hit rate (cycle 0002), blocking first-touch admission cost hit rate (cycle 0003), and ghost-ring re-admission cost a little hit rate (cycle 0006).
- Cheapening the victim search has not made puts cheaper. Sampling, stash-only restriction, and early exit each left put latency at or above the parent's level (cycles 0005, 0007). Incremental minimum caching raised put and get tails (cycle 0006).
- Counter-weight tweaks are inert or harmful. Doubling the increment and seeding new counters at 1 changed nothing measurable (cycles 0008, 0009). Sampling the increment on hits hurt hit rate (cycle 0009). Decaying counters hurt hit rate (cycle 0007).
- Stash-side bookkeeping adds cost without a payoff. Occupancy checks and maintenance on the get path raised get and put latency (cycle 0002). Recency tie-breaking added eviction cost with no hit-rate change (cycle 0003).
- Per-op latency claims have not held up in any cycle. Put and get movements repeatedly lack a mechanism in the edited code (cycles 0001 to 0009). Treat them as unconfirmed.
- Winner labels have not tracked measured results. Variants with recorded regressions or no helped dimension were still labelled Winners (cycles 0001 to 0009).

## Open questions

- Does the full-scan lowest-counter gain hold on a second, non-workload-a trace? This check has been deferred since cycle 0004 and every hit-rate result still rests on workload-a.
- Does the deferred stash-to-target promotion hold on a stash-dominant trace, as cycles 0003 and 0004 asked?
- What drives put cost, given that shortening the scan did not help and the counter update is not the get bottleneck (cycle 0009)?
- Does reordering the get lookup to skip the stash scan help or hurt stash-hit gets? The stash-hit tail has never been reported separately.
- Where does the packed layout's counter live, and what does its update cost on the hit path? Packed-layout get and promotion regressions are still unexplained.
- What is the true Winner criterion? The insights do not show which dimensions were weighted.

## Per-policy notes

- Frequency-aware victim selection (full-scan, saturating, non-decaying counter): the only policy with a repeated retention gain. Keep the full scan. Avoid sampled, two-choice, stash-only, decayed, or incrementally cached variants. Its put-path cost is real but its source is unidentified.
- Insert and promotion placement (MRU insert, fullness-gated insert, admission delay, ghost ring): every variant that blocked, delayed, or displaced entries lost hit rate. Keep new keys able to enter the target bucket early.
- Stash-to-target promotion (deferred): hit-rate neutral to positive when its first-hit state is cheap. Only tested on workload-a. Build on it only after a stash-dominant check.
- Recency tie-breaking: no effect on hit rate. Drop.
- Record layout (packed or repacked): does not change eviction but has not yielded a clean latency win. Any packing needs its hot-path decode and counter update measured before it is built on.
