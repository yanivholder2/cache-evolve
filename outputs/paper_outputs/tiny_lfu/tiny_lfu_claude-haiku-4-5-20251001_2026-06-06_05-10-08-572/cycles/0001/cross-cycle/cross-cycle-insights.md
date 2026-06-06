# Cross-Cycle Insights

## Recurring wins

- **Frequency estimation accuracy outweighs per-operation latency optimization**: v001's wider sketch improved both hit-rate and per-op latency simultaneously despite increased memory; v002's latency gains came at the cost of hit-rate and tail performance. The evidence suggests decision quality (admission/eviction) is the bottleneck, not hash computation speed.
- **Memory investment in collision avoidance multiplies returns**: Doubling sketch width (v001) yielded compounding benefits across metrics, indicating that hash collision reduction on Zipfian workloads justifies the metadata cost.
- **Admission policy and estimation accuracy are orthogonal tuning axes**: v003's throughput gains operated independently of sketch structure, showing that operational efficiency can be purchased without sacrificing frequency-based decision quality.
- **Tail latency is a more sensitive fitness signal than median**: v002's p50 improvement masked p99 degradation and put-operation slowdown, revealing that poor frequency estimates create adverse cache states that harm the tail disproportionately.

## Recurring failures

- **Aggressive estimation cost reduction backfires on Zipfian workloads**: v002's sketch depth cut improved get p50 but cascaded into hit-rate loss and eviction thrashing, proving that latency wins at the expense of accuracy are false economies in decision-heavy workloads.
- **Frequency accuracy cannot be traded away for operational gains alone**: Poor estimates drive worse admission decisions regardless of how fast admission itself executes, making the admission barrier relaxation (v003) insufficient as a standalone lever without paired accuracy investment.

## Open questions

- **Do width and admission policy synergize or conflict?** v001 and v003 moved in opposite directions on hit-rate; pairing them might find a balanced sweet spot or reveal diminishing returns.
- **Where is the lower bound on sketch depth before accuracy collapse?** v002 went to depth 2; intermediate values (3–4) may preserve hit-rate while extracting some latency gains.
- **How do Zipfian skew characteristics affect the frequency-vs.-latency tradeoff?** On less skewed workloads, does operational latency become the limiting factor, shifting the optimal tuning strategy?
- **Can sketch dimensions adapt to detected workload properties in flight?** Workload heterogeneity may justify dynamic tuning of width/depth based on observed access patterns.

## Per-policy notes

- **v001 (Sketch width increase)**: Wider counters reduced hash collisions on high-cardinality Zipfian traces, yielding simultaneous gains in hit-rate and per-op latency. Memory cost justified by multiplicative returns.
- **v002 (Sketch depth reduction)**: Improved get p50 and reduced per-operation work, but sacrificed frequency estimation accuracy, driving worse eviction decisions, higher p99, and put-operation degradation. Tail latency and admission quality were the true cost.
- **v003 (Admission barrier relaxation)**: Delivered ~17% latency improvement via reduced admission overhead, operating orthogonally to frequency estimation. Trades hit-rate for throughput; benefit depends on workload's admission:compute ratio.
