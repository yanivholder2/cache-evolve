# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive promotion thresholds** (cycle 0003 v003, confirmed superior in cycles 0004–0005): The only mechanism to achieve latency gains without sacrificing hit-rate. Loosening the gate when main queue has spare capacity (freq > 1) and tightening under contention (freq > 3) decouples the two metrics. This remains unbeaten across subsequent attempts to refine or extend it.

- **Strict admission filtering for latency** (cycles 0001–0002): Tight frequency thresholds (freq > 2, freq > 3) consistently reduce main-queue contention and improve operation latency by 28–66%. The mechanism is reliable and works across variants, though always paired with hit-rate cost.

- **Threshold-level adaptation is overhead-free** (cycles 0003–0005): Admission decisions that respond to queue occupancy can be evaluated without per-operation instrumentation, unlike structural changes. This is why cycle 0003's occupancy-responsive thresholds succeeded where cycle 0004–0005's per-operation sizing adjustments incurred latency overhead.

## Recurring failures

- **Ghost-queue structural separation for frequency preservation** (cycles 0003–0005): Isolating frequency tracking into side structures has failed consistently. Cycle 0003's warm-ghost queue didn't translate to hit-rate recovery (items overflowed faster than re-accessed). Cycle 0005's approaches (loosened retention, full preservation without decay) corrupted admission signals—ghost entries accumulated inflated hotness that conflated evicted items with genuinely hot ones. The problem is tight coupling: retention policy, decay schedule, and re-admission weight must all be calibrated together, and tuning any one lever in isolation breaks the system.

- **Structural per-operation overhead as a hit-rate mechanism** (cycles 0004–0005): Any technique requiring per-operation tracking (dynamic queue sizing, frequent resizing checks) introduces measurable overhead that forces new latency–hit-rate trade-offs. Cycle 0004's v003 recovered hit-rate through dynamic sizing but incurred +36ns get latency and +86ns put p99. Cycle 0005's v002 proved that removing adjustment frequency recovered latency but lost hit-rate. Occupancy-awareness works only at the decision level, not the structural level.

- **Frequency distribution as an occupancy signal** (cycle 0004): Percentile-based thresholds (75th percentile) proved less robust than queue occupancy. They require workload-specific calibration, are sensitive to access-pattern shifts, and add sampling overhead. Occupancy directly reflects queue state; frequency distribution does not.

- **Simple queue-sizing tuning as a hit-rate recovery lever** (cycles 0002, 0004): Expanding queue ratios or adjusting capacities without addressing the admission threshold fails to recover hit-rate. The promotion threshold is the binding constraint—larger windows don't help if the gate is too strict. Queue structure is secondary.

## Open questions

- **Can ghost-queue decay be properly calibrated?** Cycles 0004–0005 struggled to balance retention with signal fidelity. Cycle 0005 asked whether stricter decay semantics (decay to -2 or -3) could preserve re-access candidates without corrupting admission decisions. This remains unexplored.

- **Is cycle 0003 v003 truly a local optimum, or can fine-grained occupancy-threshold tuning escape it?** Cycle 0005 suggested exhausting parameter space (e.g., occupancy thresholds at 0.4, 0.6, 0.8 rather than 0.5, 0.8) before pivoting to new mechanisms. No follow-up has tested this.

- **Could fundamentally different admission mechanics avoid the current frontier?** Recency-weighted heuristics, bimodal thresholds, or decay-based ranking have not been tested. All cycles converged on frequency-based gating; alternatives remain unexplored.

- **Does occupancy-aware threshold adaptation create latency oscillation near occupancy boundaries?** Cycle 0003 raised whether transitions between thresholds (e.g., at 50%, 80%) are smooth or introduce spikes. No stress-testing across workload phases has verified stability.

## Per-policy notes

- **Promotion threshold policies**: The critical lever. Fixed thresholds (freq > 2, freq > 3) improved latency consistently but locked in hit-rate cost (cycles 0001–0002). Occupancy-responsive thresholds (freq > 1 when spare, freq > 3 under load) broke the trade-off by allowing dynamic gates (cycle 0003, confirmed superior in cycles 0004–0005). All subsequent attempts to refine threshold tuning (decay schedules, percentile signals, dynamic sizing coupled to thresholds) either incurred overhead or corrupted admission signals.

- **Ghost-queue management**: Tested across cycles 0003–0005 with consistent failure. Structural separation (warm-queue isolation) provided no hit-rate benefit. Frequency decay showed promise for put latency (cycle 0004) but required over-aggressive filtering to avoid overhead, degrading hit-rate. Preservation without decay (cycle 0005) collapsed signal fidelity. Ghost queues appear unsuitable for frequency-preservation recovery within the current framework.

- **Queue sizing and structure**: Explored as a secondary lever in cycles 0001–0005. Simple ratio adjustments (25% → 30% small queue) didn't recover hit-rate and increased churn (cycle 0002). Dynamic sizing adapted to occupancy (cycles 0004–0005) improved hit-rate but introduced measurable per-operation overhead that traded latency gains. Static sizing paired with occupancy-responsive thresholds (cycle 0003) remains the most balanced approach.

- **Eviction timing**: Tested in cycle 0002 (lazy eviction, deferred main-queue eviction). Had no meaningful effect on hit-rate and didn't improve latency as hypothesized. Eviction frequency is not the constraint; admission filtering is.
