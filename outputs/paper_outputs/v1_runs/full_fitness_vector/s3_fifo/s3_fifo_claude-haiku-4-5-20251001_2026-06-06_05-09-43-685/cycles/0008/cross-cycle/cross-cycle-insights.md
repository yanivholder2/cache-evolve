# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive thresholds decouple latency and hit-rate**: Cycle 0003's v003 demonstrated that promotion decisions keyed to queue occupancy—not fixed thresholds—can achieve latency improvements without hit-rate erosion. This baseline has survived validation through cycles 0004–0008 as the most balanced design.

- **Cold-path computation beats hot-path overhead**: Cycles 0001–0008 consistently show that eviction-time filtering and decisions (infrequent) incur zero hot-path cost, while per-operation occupancy tracking, branching, or adjustment on critical paths (especially small-queue puts) introduce measurable latency penalties that offset hit-rate gains. Placement of computation matters as much as its logic.

- **Pre-computed ghost-queue state with adaptive re-admission signals**: Cycles 0006–0007 validated that storing frequency or metadata in ghost entries, coupled with occupancy-responsive or signal-blended re-admission formulas (not fixed arithmetic), enables both latency and hit-rate improvements. The key is the re-admission signal, not the architecture alone.

- **Admission filtering tightness is the primary latency lever**: Cycles 0001–0007 confirm that stricter promotion gates consistently deliver latency gains. The binding constraint is not how the filter is tuned, but whether it's applied—all relaxations (attempted in cycles 0004–0008) reintroduced latency regression.

## Recurring failures

- **Fine-grained parameter tuning on established designs plateaus quickly**: Cycles 0005–0006 repeatedly showed that adjusting occupancy breakpoints (0.4 vs. 0.5, 0.7 vs. 0.8), decay windows (-1 vs. -2 vs. -3), or promotion ranges within existing frameworks yields negligible improvement or regression. The design's parameters were near-optimal; further tweaking explores noise.

- **Ghost-frequency preservation without decay discipline corrupts admission signals**: Cycle 0005's v001 and v003 attempted to retain more frequency information in ghost to recover hit-rate, but without strict decay rules, evicted items accumulated inflated hotness that corrupted main-queue decisions. The coupling between retention policy, decay schedule, and re-admission weight is too tight for independent tuning.

- **Per-operation occupancy-aware structural mechanics**: Cycles 0004–0008 showed that computing occupancy ratios or executing conditional branches on eviction or put operations introduces overhead that forces new trade-offs. Queue resizing (0004 v003) and occupancy-gated retention (0008 v003) both degraded latency despite targeting hit-rate recovery. Occupancy works only as a threshold signal, not as a per-operation mechanical adjustment.

- **Recency as a primary re-admission signal is insufficient**: Cycles 0007–0008 confirmed that entry position in ghost is cheaper than frequency but too weak a predictor of re-access likelihood in this workload. Recency delivers latency gains at measurable hit-rate cost with no clear path to recovery through tuning.

- **Frequency-decay variants on already-lean baselines reintroduce cost**: Cycle 0007's v001 and cycle 0008's v001 attempted to add frequency preservation or decay logic to lean designs but reintroduced per-operation overhead that contradicted the lean design philosophy, without delivering offsetting hit-rate gains.

## Open questions

- **Can adaptive re-admission formulas be further refined?** Cycles 0006–0007 showed that occupancy-responsive boost scaling (v002) works, but "conservative range" and "suboptimal weighting" suggest tuning margin exists. Could frequency-distribution-aware scaling or occupancy-gated signal weighting recover more hit-rate while retaining latency?

- **Is there a ghost-frequency decay depth that balances both metrics?** Multiple cycles (0004, 0005, 0006) flagged decay windows (-1, -2, -3) as underexplored. Could a specific depth (e.g., -2.5, or adaptive decay based on access velocity) find a sweet spot between latency and hit-rate?

- **Can multi-signal re-admission be tuned to outperform single signals?** Cycle 0008's v002 combined frequency and recency with equal weighting and conservative boost; does adaptive weighting (e.g., occupancy-responsive mixing of frequency and recency) or frequency-distribution-aware normalization unlock both latency and hit-rate gains?

- **Has a Pareto frontier been reached, or do unexplored mechanisms remain?** Cycles 0005–0008 repeatedly ask this. Current search space is dense in threshold and signal variants; are there fundamentally different admission mechanics (e.g., time-decayed frequency ranking independent of eviction, probabilistic re-admission) that could improve both metrics?

- **Can hot-path overhead be eliminated through deferred or batched computation?** Cycle 0008 notes that occupancy sensitivity on eviction-time decisions is free; could occupancy computation be amortized (computed once per N evictions) or moved entirely off critical paths while retaining its decision-quality benefit?

## Per-policy notes

- **Admission filtering via frequency thresholds**: Fixed thresholds (freq > 2, freq > 3) reliably improve latency but degrade hit-rate. Occupancy-responsive thresholds (freq > 1–3 scaled by queue occupancy) successfully balance both metrics; this mechanism has not been surpassed in 8 cycles.

- **Ghost-queue management**: Simple structural separation (cycle 0003 v001) fails because frequency information must be embedded in critical paths, not side structures. Pre-computed state (storing frequency in ghost entries) succeeds only when paired with calibrated re-admission signals. Filtering ghost at eviction time is overhead-free; filtering on re-admission is not.

- **Queue resizing and structural adaptation**: Static sizing with occupancy-responsive thresholds is more efficient than dynamic resizing. Per-operation occupancy tracking for structural decisions introduces hot-path overhead that forces latency-hit-rate trade-offs. Occupancy awareness should remain at the decision/signal level, not the structural/mechanics level.

- **Re-admission signals**: Fixed arithmetic (e.g., ghost_freq + 3) overshoots some decay trajectories and undershoots others, failing to adapt to workload state. Occupancy-responsive scaling (boost = baseline + occupancy_factor) succeeds; recency alone fails. Signal combination (frequency + recency) is architecturally sound but brittle without proper weighting—likely requires occupancy-responsive adaptation or frequency-distribution awareness.

- **Occupancy as a design signal**: Zero-cost when used for threshold selection (cycle 0003); expensive when used for branching or per-operation mechanics (cycle 0008). Should be confined to decision-time choices (which threshold to use, which re-admission formula to apply), not control-flow or adjustment logic on hot paths.
