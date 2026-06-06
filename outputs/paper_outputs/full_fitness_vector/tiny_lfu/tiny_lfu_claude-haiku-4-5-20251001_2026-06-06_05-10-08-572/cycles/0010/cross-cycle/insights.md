# Cross-Cycle Insights

## Recurring wins

- **Frequency estimation accuracy is the primary performance lever.** Cycles 0001–0002 established that memory spent on collision avoidance (width expansion) yields multiplicative returns across hit-rate and latency simultaneously, while per-operation cost reductions trade off accuracy. Frequency-aware admission and eviction decisions consistently outperform symmetric heuristics, validating this axis as the system's fundamental constraint.

- **Minimal, targeted eviction-pool expansion is high-leverage and efficient.** Cycles 0006–0010 converged on N=2 and N=3 as sweet spots: N=2 delivered 30% p99 latency reduction with negligible hit-rate cost (cycle 0006 v001); N=3 extended hit-rate gains (+0.37%) at tolerable latency cost (cycle 0009 v001). N=1 regressed on both metrics (cycle 0007), N=4 collapsed to diminishing returns (cycle 0010), establishing a tight, stable frontier.

- **Workload-semantic policies outperform symmetric thresholds.** Cycle 0009–0010 validated that asymmetric admission (update-preferential) improves latency by streamlining the hot path without hit-rate penalty, while symmetric rules (absolute frequency floors, relaxed gates) consistently fail. Policies that leverage entry-type or access-history structure are more robust than global heuristics.

- **Simple parameter tuning outperforms complex state-management refactoring.** Cycle 0007 v003 reduced width by a constant, delivering 17–18% latency gains; cycle 0007 v002 added caching logic to reduce function calls and regressed 42–139% due to microarchitectural penalties. Minimal surface area (fewer variables, branches) matters more than algorithmic complexity reduction at this operational scale.

- **Sketch geometry optimization delivered consistent, reliable gains within feasible bounds.** Cycles 0001–0007 progressively reduced sketch dimensions (depth 4→3, width 4→3) with clear latency and hit-rate wins, establishing depth-3 width-3 as a durable baseline. Cycle 0008's exhaustive boundary tests confirmed this is a hard frontier; narrower geometry triggers 1.2–1.6% hit-rate loss and catastrophic tail pathology.

## Recurring failures

- **Periodic sketch decay is fragile and non-generalizable across parameter variations.** Cycle 0003 v003 claimed simultaneous hit-rate and latency improvements; cycles 0004 and 0006 failed to reproduce those gains. Decay adds unconditional operation-count checking overhead, exhibits no smooth parameter gradient (cycle 0004 v001–v002 showed neither aggressive 250k nor conservative 1M intervals matched the claimed 500k optimum), and has poor composability (cycle 0004 v003 created tail-latency spikes when paired with wide geometry). Mechanism is workload-specific or a statistical artifact.

- **Composition of multiple overhead-adding mechanisms amplifies failure modes.** Cycle 0004 v003 combined decay with wide geometry and created 738 ns p99 get latency spikes; cycle 0010 v001 combined asymmetric admission with N=3 victim selection and reversed expected hit-rate improvements. Mechanisms interact through shared admission/eviction pressure in unpredictable ways. Treating them as independent levers leads to cascade failures.

- **Symmetric absolute-frequency thresholds create tail-latency pathology.** Cycles 0002, 0005, and 0009 consistently found that relaxing admission barriers (>= instead of >, absolute floors like freq >= 2) improve median metrics while introducing cliff-like p99 degradation. The strict `incoming_freq > victim_freq` condition encodes workload-specific temperature distribution; marginal threshold relaxations admit items that trigger eviction chains and cascade failures.

- **Hash function quality optimization does not correlate with hit-rate on this workload.** Cycle 0005 v002 tested XOR-combined hashing for row decorrelation (a standard technique for count-min sketches) and regressed 36% on get latency, 27% on put latency, with no hit-rate improvement. Baseline single-seeded hash function is adequate; collisions are not the primary bottleneck, or the workload has favorable collision properties.

- **Aggressive victim-pool expansion yields superlinear latency cost.** Cycles 0005–0006 showed that increasing candidate-pool size from 2 to 5 caused 96%+ tail-latency spikes on put operations. Scanning overhead under eviction pressure compounds nonlinearly. N=2 or N=3 represents the viable frontier; deeper pools are systematically rejected across multiple cohorts.

- **Depth reduction below 3 triggers hard boundary failures.** Cycle 0008 showed depth-2 causes 1.2% hit-rate loss and 141% p99 latency spike—a phase transition rather than gradual degradation. The boundary is coupled to sketch width (both must stay at 3 or wider), suggesting collision probability requirements in the Zipfian distribution create a non-negotiable minimum.

## Open questions

- **Is there a workload-adaptive parameter schedule responsive to phase transitions?** Sketch decay was hypothesized to capture phase sensitivity but failed due to overhead. Could admission thresholds, victim-pool sizes, or admission criteria adapt in real time based on cache miss rates or hit-rate trends? All evidence is from static parameters on a single trace.

- **Can asymmetric admission be extended to distinguish more entry classes (recency, cost, frequency distribution)?** Cycle 0009–0010 validated update-preferential distinction. Could more granular policies (e.g., recently accessed items always admitted, cold items strictly gated) further improve the latency-hit-rate tradeoff?

- **Why is the depth-3, width-3 boundary a hard discontinuity rather than gradual degradation?** Cycle 0008's systematic tests indicate a phase transition in collision probability tied to sketch geometry, but the mechanism is unclear. Is this fundamental to count-min sketches under Zipfian distributions, or workload-specific?

- **Can frequency-aware eviction be implemented with lower microarchitectural cost?** Cycle 0007 v002 failed because state-management overhead outweighed algorithmic savings. Could vectorized sketch queries, cache-line aware memory layout, or lazy evaluation reduce per-operation cost without sacrificing frequency awareness?

- **Do patterns (frequency importance, sketch geometry bounds, N=3 eviction optimum) generalize to other workloads?** All evidence is from a single Zipfian trace. Robustness across diverse Zipfian skews, cache sizes, and workload characteristics is unvalidated. Phase-shifting or bursty access patterns may break assumptions.

- **Is there an orthogonal admission lever independent of frequency estimation?** Cycles focused on frequency-based gates, but alternative criteria—recency heuristics, cost-benefit models, access-pattern classification, or semantics beyond update/new distinction—remain unexplored.

## Per-policy notes

**Eviction (LRU + frequency-aware victim selection):**
- N=2 frequency-aware selection is the stable baseline, delivering 30% p99 put latency reduction with minimal hit-rate cost (cycle 0006).
- N=1 (pure LRU) regressed on both metrics, confirming frequency awareness is load-bearing (cycle 0007).
- N=3 extends hit-rate gains (+0.37%) at acceptable latency cost; N=4 collapsed to +0.03% (diminishing returns, cycle 0010).
- Frequency-aware selection prevents cascading evictions and admission storms; removing it harms both latency and hit-rate.

**Admission (frequency-based gating):**
- Asymmetric policies (update-preferential) improve latency by streamlining hot-path frequency comparisons, independent of hit-rate (cycle 0010 v003).
- Strict `incoming_freq > victim_freq` threshold is a local optimum with no smooth gradient; relaxation (>=, -1 offset) consistently backfires on tail metrics.
- Absolute frequency floors (e.g., freq >= 2) improve median metrics but create outlier-case p99 spikes (cycle 0009 v002).

**Sketch geometry (depth, width):**
- Depth-3, width-3 is the locked-in baseline following cycle 0008's exhaustive boundary validation.
- Width is higher-leverage than depth: width-4→3 delivered 17–18% latency gain (cycle 0007); depth-4→3 achieved <0.3% hit-rate loss (cycle 0002).
- Geometry narrower than 3 in either dimension triggers phase-transition failures (1.2–1.6% hit-rate loss, 141% p99 spikes).

**Abandoned mechanisms:**
- Sketch decay (cycles 0003–0006): Added unconditional overhead, failed to generalize across parameter variations, poor composability.
- Hash function optimization (cycle 0005): No hit-rate improvement; baseline hash sufficiency suggests collisions are not the bottleneck.
- State-management caching (cycle 0007): Microarchitectural penalties outweighed algorithmic savings; simple parameter tuning is preferable to code refactoring at this scale.
