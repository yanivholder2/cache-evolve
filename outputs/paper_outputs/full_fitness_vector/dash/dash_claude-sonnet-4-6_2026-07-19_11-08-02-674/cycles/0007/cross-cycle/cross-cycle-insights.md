## Recurring wins

- **MRU-end insertion** has been a stable anchor since Cycle 1 — a single placement flip improved both hit-rate and per-op time simultaneously and has held through all subsequent cycles without regression.
- **Struct field removal (timestamp)** delivered a dual dividend (smaller per-item footprint, eliminated syscalls) with zero hit-rate cost; similar overhead audits are likely to yield further free wins.
- **Orthogonal improvements stack additively** when they touch non-overlapping subsystems — confirmed for insertion position × struct compaction (Cycle 2) and for eviction selection × promotion gating (Cycles 5–6).
- **Vacancy-gated frequency promotion (≥2 threshold)** has been regression-free since Cycle 3 and survived stacking through Cycle 6; gating promotion fires on tier vacancy is the structural fix that eliminates cascade evictions by construction.
- **Two-chance clock eviction** (Cycle 6) solved the long-standing O(n) scan latency problem: rotating warm items to MRU end and stopping at the first cold item delivers frequency-aware victim selection with amortized O(1) cost and improved latency on a Zipfian workload.

## Recurring failures

- **Threshold-only calibration without a vacancy guard** consistently softens but cannot cure the latency regression from promotion cascades; the root cause is architectural (firing into a full tier), not parametric.
- **Stash-bypass on the MRU-end base** reversed from a win (LRU-end base, Cycle 2) to a regression (MRU-end base, Cycle 3) because MRU insertion makes target saturation the common case, over-triggering the bypass and converting probing into a spillover sink.
- **Joint-condition mechanisms that require two rare simultaneous events** (e.g., target-full AND probing-vacant at stash-hit time) fire too infrequently on this workload to produce a measurable effect — confirmed inert in Cycle 4.
- **Counter decay** is antagonistic to frequency-gated promotion on a stationary workload: it suppresses counters below the promotion threshold and adds a second O(n) scan at eviction time, doubling median latency (Cycle 6).
- **Merging clock eviction with frequency-gated promotion via the shared `lfu_counter`** fails structurally: the clock requires counter-zero cold items to exit early; the promotion gate requires non-zero counters to identify warm items. When the bucket is counter-rich, the clock degrades to O(k·n) rotation and erases its latency advantage (Cycle 7).
- **Vacancy-gated probing re-entry** is structurally safe but persistently thin at high cache occupancy (Cycles 5, 6, 7) — the joint firing condition is too rare under the Zipfian steady-state where hot items cycle through probing continuously.

## Open questions

- Can the two-chance clock and frequency-gated promotion coexist if their signals are decoupled — e.g., a separate boolean "chance bit" for the clock that leaves `lfu_counter` untouched for promotion gates?
- Can the confirmed hit-rate signal from counter-increment on target access (H03) be captured cheaply by gating the increment only when an item is near eviction risk, avoiding the common-path overhead that tripled median latency in Cycle 7?
- Are there additional struct fields or per-operation overhead sources beyond `timestamp` still worth auditing in the current baseline?
- Does counter decay become net-positive on a non-stationary workload where stale-popularity drift is a real problem, and can a firing-rate guard make it safe to combine with frequency gates?
- At what cache-to-working-set ratio does the probing re-entry vacancy gate fire often enough to produce a measurable hit-rate contribution?

## Per-policy notes

- **Positional LRU (baseline)**: MRU-end insertion was a direct improvement from Cycle 1 and has been the default since; no further LRU-order wins have appeared.
- **LFU-minimum eviction (O(n) scan)**: Independently confirmed as a hit-rate lever in Cycles 5–6 and additive with vacancy-gated promotion, but consistently inflated put p99 due to scan cost. Superseded by the two-chance clock as the preferred frequency-aware eviction strategy.
- **Two-chance clock eviction**: Structurally improved all three measured dimensions in Cycle 6 from baseline. Incompatible with frequency-gated promotion when sharing `lfu_counter` — clock decrements corrupt the promotion signal (Cycle 7). Decoupling the chance bit from the frequency counter is the unresolved prerequisite for composing these two mechanisms.
- **Frequency-gated vacancy-guarded promotion (≥2)**: Regression-free hit-rate lift since Cycle 3; ≥2 appears near-optimal for this workload (≥3 produced a net loss in Cycle 4). Survives stacking with LFU-minimum eviction and probing re-entry (Cycles 5–6) but conflicts with both counter decay and the two-chance clock when `lfu_counter` is shared.
- **Counter decay**: Single test in Cycle 6; harmful on a stationary distribution — doubles median latency and suppresses the frequency signal. Should be reserved for non-stationary workloads where popularity shifts are a real operational concern.
