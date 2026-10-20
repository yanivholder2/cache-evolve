# Cross-Cycle Insights

## Recurring wins

- **Structural constraints on eviction search are the critical latency lever**: Both Cycle 0002 and 0003 confirmed that pathological O(N) eviction latency spikes from Cycle 0001 disappear when search is bounded (min(capacity/4, 256) nodes) or pre-organized (frequency buckets). The eviction *data structure* choice matters more than the selection strategy itself.

- **Workload-aware mode selection outperforms uniform strategy**: Cycle 0003's adaptive bucketing (V003) decisively beat uniform approaches (V001, V002) by selecting expensive logic conditionally based on saturation. It achieved 34% get p99 reduction on underutilized workloads and preserved bucketing capability for high-pressure scenarios, showing that static design choices lose to dynamic adaptation.

- **Hit rate is stable within the bounded-window framework**: Across Cycles 0002 and 0003, all variants using bounded windows maintained near-identical hit rates (~0.6904) regardless of internal selection strategy (binary threshold, multi-level ranking, or adaptive bucketing), suggesting the window size and structure matter far more than fine-tuning selection criteria.

## Recurring failures

- **Deferring work shifts cost but does not eliminate it**: Cycle 0003's lazy-batch approach (V001) moved bucket updates from get-time to eviction-time, reducing get p99 by 5% but spiking put p99 by 64%. The underlying computational cost cannot be eliminated, only deferred—and deferral concentrates variability in a different operation.

- **Linear eviction scans and over-fitted metadata are pathological**: Cycle 0001 established that selective eviction (V002, V003) improved hit rate but required O(N) scans, causing 20× put p99 spikes. Cycle 0002 showed hybrid approaches (zone+frequency without bucketing) exacerbated this. Metadata overhead (frequency counters, resurrection maps) is a universal tax even on underutilized caches.

- **Multi-level ranking granularity lacks diminishing returns in bounded windows**: Cycle 0003's V002 added multi-level frequency ranking within the bounded window, improving hit rate marginally while degrading put p99 by 38%. This indicates binary thresholds (hot vs. cold) already capture most selectivity benefit within small scan windows; finer granularity adds decision complexity without proportional gain.

## Open questions

- **Will V003's mode-switching remain rare under sustained pressure?** The validated workload was underutilized and never triggered high-saturation bucketing activation. Longer traces, mixed workloads, or sudden saturation spikes may cause frequent transitions, eroding the cost-benefit of dual-mode infrastructure.

- **Can frequency decay tuning suppress V002's normal-case latency overhead?** Cycles 0002 and 0003 showed frequency bucketing improves hit rate and tail latency but increases median latencies. Better decay mechanisms or adaptive thresholds might move the Pareto frontier, but this remains untested.

- **How do these strategies generalize to non-Zipfian or adversarial access patterns?** All cycles tested on a single workload. Robustness under sudden pattern shifts, uniform access, or correlated bursts remains unvalidated.

- **Can saturation and mode-transition decision overhead be reduced via sampling or lazy evaluation?** V003 adds per-operation saturation tracking and mode-logic checks; periodic sampling or amortized transitions might lower the constant-factor cost of the adaptive layer.

## Per-policy notes

- **Bounded-window (V001 core design)**: Latency-friendly, simple, stable hit rate across contexts. Avoids pathological spikes through structural search constraint. Validated across Cycles 0002 and 0003; trade-off is loss of fine-grained selectivity, but appears sufficient for Zipfian workloads.

- **Frequency bucketing (V002)**: Dual benefit of hit rate and tail-latency improvement, but normal-case overhead (get and put medians increase) is unavoidable. Cost-benefit depends on saturation levels and frequency distribution; unbucketed traces show poor ROI.

- **Lazy-batch updates**: Reduces one operation's latency (get) while creating spikes in another (put). Demonstrates that complexity cannot be hidden; it merely redistributes variance. Not recommended as a general approach.

- **Adaptive bucketing by saturation (V003 Cycle 0003)**: Most balanced design observed. Achieves V001's latency predictability under normal load while preserving V002's hit-rate capability for high-pressure scenarios. 72-byte metadata overhead acceptable if mode-switching stays rare, but needs stress validation.

- **Multi-level frequency ranking**: Granular ranking within bounded windows shows diminishing returns; binary (hot/cold) thresholds appear near-optimal for small scan spaces. Adds complexity without proportional latency or hit-rate payoff.
