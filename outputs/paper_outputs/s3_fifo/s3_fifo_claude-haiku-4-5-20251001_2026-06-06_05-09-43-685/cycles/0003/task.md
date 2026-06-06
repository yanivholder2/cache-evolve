# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003

## Winners

- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three winners pursued tighter admission filtering and queue resizing to reduce main-queue population and contention, achieving significant per-operation latency gains (28–66% faster operations across variants). However, all three winners trade hit-rate for these latency wins—v001 shows marginal miss impact, while v002 and v003 incur 8%+ hit-rate declines.

## Patterns

- **Latency-first trade-off**: Every variant in this cohort improves operation latency by constraining main-queue size, either through stricter promotion thresholds (v001, v002) or queue resizing (v003). The consistency suggests that reduced main-queue contention and lookup chains are a reliable latency lever.

- **Hit-rate as the cost**: The cohort reveals a fundamental tension: filtering that reduces main-queue noise also blocks items that would have been retained by looser admission. Ghost-hit re-routing (v002) and higher promotion thresholds (v001) both penalize moderately-accessed items that cannot meet stricter validation gates.

- **Workload sensitivity**: v003's unexpected latency improvement despite hit-rate decline suggests the workload benefits from smaller resident set and less eviction overhead. The three variants succeed at different severity levels, implying the trade-off frontier is workload-dependent.

## Open questions

- Can we recover hit-rate without sacrificing the latency gains—e.g., through smarter promotion heuristics or frequency-count decay?
- Do these variants cluster differently on synthetic vs. real-world access patterns?
- Is there a sweet spot where modest latency sacrifice recovers meaningful hit-rate, or is the frontier binary?

## Continuation

The cohort converges on **admission-filter tightness as a primary latency lever**. If latency is the priority metric, these variants form a solid foundation—v001 with marginal hit-rate cost is the most balanced, while v002 and v003 are extremes. **For the next cycle, either:** (1) accept the latency-first stance and optimize filter parameters further (frequency thresholds, queue sizing ratios); (2) explore frequency-decay or recency-weighted heuristics to recover hit-rate within the same tighter-filtering framework; or (3) test adaptive thresholds that adjust per workload phase to balance both metrics. Avoid looser admission filters; the cohort strongly signals they backfire.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants pushed v001's tight admission-filter strategy in different directions—raising the promotion threshold, expanding the small queue, and deferring main-queue eviction—seeking to recover hit-rate or further improve latency. All three succeeded as winners by discovering distinct latency-performance corners: v001 optimized median put efficiency at the cost of tail-latency unpredictability, v002 (surprisingly) added small-queue capacity despite reduced hit-rate and search overhead, and v003 traded get latency for put efficiency through lazy eviction. None recovered hit-rate meaningfully, confirming that the admission filter's strict threshold—not queue sizing or eviction timing—is the binding constraint.

## Patterns

- **Admission filtering remains non-negotiable**: All three hypotheses built on v001's freq > 2 (or freq > 3) threshold, and all three discovered that tweaking surrounding mechanisms (threshold level, queue ratio, eviction delay) don't fundamentally alter the hit-rate cost. The filter is the bottleneck, and tuning it in isolation (v001: freq > 3) or working around it (v002, v003) yields only indirect gains.

- **Operation-specific trade-offs dominate**: v001 improved put p50 but hurt get and put p99, revealing that median and tail latencies respond differently to the same filter. v003 flipped the trade: speeding puts while degrading gets. The cohort shows no variant improved both simultaneously—each optimizes one dimension at another's expense.

- **Simple queue-sizing doesn't recover hit-rate**: v002's expansion from 25% to 30% actually reduced hit-rate by increasing eviction churn before items could accumulate sufficient frequency. The interaction between queue capacity and promotion threshold is tighter than adjusting capacity alone can address; larger windows don't help if the threshold gate is too strict.

- **Lazy eviction addresses the wrong bottleneck**: v003's deferred main-queue eviction improved put latency by reducing eviction calls, but didn't improve hit-rate as hypothesized. Items still evicted immediately upon small-queue timeout, revealing that main-queue eviction frequency is not the constraint—admission filtering is.

## Open questions

- Can a smarter admission mechanism (frequency decay, bimodal thresholds, recency weighting) decouple hit-rate recovery from the strict promotion threshold?
- Is there a compound variation (threshold + queue ratio + eviction timing together) that balances get and put latency without sacrificing the tail?
- Do the winners' gains hold on workloads with broader frequency distributions, or is this tight filtering specific to Zipfian access?

## Continuation

The cohort unanimously points to **admission-filter tightness as a non-negotiable latency lever that carries an inherent hit-rate cost**. Further tuning the threshold level (v001), queue proportions (v002), or eviction timing (v003) moves latency around but doesn't escape the fundamental trade-off. **For the next cycle:** (1) explore smarter admission mechanisms beyond binary frequency thresholds—decay-based ranking, multi-level frequency buckets, or adaptive thresholds that respond to workload burstiness; (2) if pursuing operation-level latency balance, test compound tuning (e.g., freq > 2 with 30% queue + 85% eviction threshold) rather than isolated knobs; (3) consider whether the hit-rate penalty is acceptable as a cost of latency optimization, or prioritize frequency-preservation heuristics (ghost tracking, deferred promotion) to protect warm-but-not-hot items. Avoid further simple threshold or sizing tweaks in isolation—the search space is narrow.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
