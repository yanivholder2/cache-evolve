# Cross-Cycle Insights

## Recurring wins

- **Minimal-pool frequency-aware eviction (N=2)**: v001 in cycle 0006 achieved 30% p99 latency reduction on put operations with negligible hit-rate cost by checking only the two cheapest LRU candidates for frequency signals. This represents the practical optimum after cycles 0005–0006 validated that frequency-aware selection works but earlier variants with deeper pools (N=5) incurred 96%+ p99 latency spikes. The gain is replicable and the cost is known.

- **Moderate sketch depth reduction (depth-3 baseline)**: v002 in cycle 0002 established that reducing from depth-4 to depth-3 yields 25% per-operation latency improvement for <0.3% hit-rate loss. This baseline has remained stable across subsequent cycles and appears to be the practical lower bound; deeper subsequent experiments (v003 in cycle 0004, cycles 0005–0006) did not improve further, suggesting depth-3 is near the saturation point.

- **Width expansion on smaller, not oversized, sketches**: v001 in cycle 0001 doubled width and improved both hit-rate and latency simultaneously, establishing width as a high-ROI tuning axis. Subsequent cycles (v001 in cycle 0003) revealed the benefit is cache-size and baseline-specific; expansion on an already-optimal baseline (depth-3) backfires due to cost.

## Recurring failures

- **Admission threshold relaxation**: v001 in cycle 0002, v002 in cycle 0003, and related attempts consistently fail. Relaxing the strict `incoming_freq > victim_freq` inequality to `>=` or `>= victim_freq - 1` admits borderline items that displace higher-value residents. The threshold encodes workload-specific temperature distribution knowledge that marginal relaxations cannot overcome. No smooth gradient exists.

- **Sketch decay as a universal lever**: v003 in cycle 0003 appeared to improve both hit-rate and latency simultaneously, but subsequent validation (cycle 0004, v001–v003; cycle 0006, v002) failed to reproduce the benefits. Decay adds O(depth × width) overhead every N operations or probabilistically per-operation, and the conditional cost is visible in latency even when hit-rate gains are marginal or absent. The mechanism is fragile and likely workload-specific or a statistical artifact of the cycle 0003 trace.

- **Hash function decorrelation**: v002 in cycle 0005 tested XOR-combined hashing to reduce collision-induced estimation errors. The variant regressed on both hit-rate and latency, indicating collisions are not a bottleneck on this workload and that the baseline's single `DefaultHasher` with simple seed variation already achieves sufficient independence.

- **Stacking overhead-adding mechanisms**: v003 in cycle 0006 combined decay (500k interval) with N=2 frequency-aware selection, hypothesizing synergy. Instead, both mechanisms' overheads compounded: put p99 latency spiked 1217 ns above baseline, burying any hit-rate gains. Composing multiple tuning levers amplifies worst-case costs without demonstrated synergy.

- **Sketch geometry expansion beyond depth-3, width-4**: v003 in cycle 0002 (depth-6) and v001 in cycle 0003 (width-8) both aimed to reduce collision-induced estimation errors or recover hit-rate. Both paid latency costs (11% and 60%, respectively) without commensurate hit-rate gains, confirming that the sketch bottleneck has shifted from collision density (geometry) to computational overhead (eviction, decay) or workload phase characteristics (admission).

## Open questions

- **Was cycle 0003's decay breakthrough real or statistical?** All four subsequent decay experiments (cycle 0004 v001–v003, cycle 0006 v002) failed to reproduce hit-rate and latency co-benefits. Is decay beneficial only under narrow phase-transition characteristics present in that workload trace, or was the result a testing artifact?

- **What is the lower bound on frequency-aware eviction pool size?** v001 in cycle 0006 (N=2) succeeded; direct comparison to N=1 (pure LRU head, zero frequency awareness) and N=3 would bracket the frontier and clarify whether minimal frequency signals are necessary or if recency alone is sufficient.

- **Does hash function quality truly not matter, or are baseline collisions favorable by accident?** v002 in cycle 0005 showed decorrelation adds cost with no hit-rate gain. Is the workload's key distribution simply favorable to simple seeded hashing, or are there subtler collision patterns that a more sophisticated hash strategy could exploit?

- **Can frequency-aware eviction overhead be reduced through algorithmic changes?** Prior variants paid latency variance (scanning cost); could lazy evaluation, cached frequency estimates, or amortized computation reduce the cost without sacrificing hit-rate gains?

- **What is the next optimization frontier beyond eviction policy?** Sketch geometry has plateaued (depth-3, width-4 is stable), eviction has reached diminishing returns (N=2 is efficient), and decay adds no net benefit. Do orthogonal levers exist (workload-adaptive parameter schedules, admission policy micro-tuning with cost awareness, cache-size variations, or hot-path instruction profiling) that offer material gains?

## Per-policy notes

- **Sketch geometry (width/depth trade)**: Depth-3, width-4 is the stable baseline across cycles 0002–0006. Width expansion pays off on smaller bases (cycle 0001) but becomes expensive at larger scales (cycle 0003). Depth expansion shows diminishing returns; collision reduction is not the limiting factor at depth ≥ 3. Geometry tuning has saturated.

- **Admission threshold**: Strict `>` inequality is workload-optimized and brittle. No gradient exists toward permissiveness; relaxation uniformly fails (cycles 0002, 0003). Threshold tuning should be deprioritized unless paired with workload-adaptive context (e.g., phase-aware switching).

- **Eviction policy**: LRU baseline is stable and latency-predictable. Frequency-aware victim selection improves hit-rate but at latency variance cost; the sweet spot is minimal pool size (N=2), not deeper candidate evaluation. Pools larger than N=2 pay superlinear scanning cost for sublinear hit-rate recovery. Broader frequency-aware policies (median filtering, deep scanning) are disqualified by tail-latency spikes.

- **Decay and temporal freshness**: Periodic or probabilistic count resets add unconditional overhead (op_count checks, conditional branches) without reproducible benefit. Decay appears orthogonal to or antagonistic with geometry-based and eviction-based tuning. Unless implemented with zero-cost mechanisms, decay should be deprioritized.

- **Hash function quality**: Baseline `DefaultHasher` with simple seed variation is sufficient on this workload. Decorrelation techniques add cost without hit-rate gain; collision reduction is not a bottleneck.
