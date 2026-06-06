# Cycle 0004 Insight

## Cohort outcome

This cycle tested three mechanisms to refine v003's occupancy-responsive success from cycle 0003: frequency decay in the ghost queue (v001), frequency-distribution percentile thresholds (v002), and dynamic small-queue sizing (v003). The two winners diverged sharply: v001 achieved significant put-latency gains by aggressively filtering the ghost queue, but sacrificed hit-rate; v003 recovered hit-rate through dynamic occupancy-aware queue sizing, but incurred latency overhead. v002's attempt to use frequency distribution as a threshold signal failed, proving less robust than occupancy-based signaling. The cohort reveals a critical constraint: occupancy-awareness successfully decouples metrics at the threshold level (cycle 0003), but applying it to structural queue sizing introduces per-operation overhead that forces new trade-offs.

## Patterns

- **Occupancy-awareness scales poorly to structural changes**: v003's dynamic small-queue sizing successfully improved hit-rate (0.7162→0.7248) by adapting capacity during load phases, but the occupancy tracking and resizing logic on every put operation degraded both get and put latencies (get p50 +36ns, put p99 +86ns). Threshold adaptation (cycle 0003) was overhead-free; queue structure adaptation is not.

- **Ghost-queue filtering requires calibration, not just filtering logic**: v001's frequency decay successfully reduced ghost-queue overhead and accelerated puts (p50: 577→203ns), but the filtering was too aggressive—items were ejected before accumulating sufficient re-access probability. The mechanism shows promise (frequency preservation embedded in the critical path, not structurally separated as in cycle 0003's v001), but the decay rate or retention threshold needs loosening to preserve more ghost entries.

- **Frequency distribution is a less stable signal than occupancy**: v002's 75th percentile threshold proved too conservative for the workload, reducing hit-rate and adding sampling/percentile-computation overhead to critical paths. Unlike occupancy (which directly reflects queue state), frequency distribution requires workload-specific calibration and is sensitive to access-pattern shifts. Occupancy remains the more robust observable signal.

- **The cohort discovered an unresolved latency-hit-rate scaling problem**: Two winners each optimized one metric at the other's expense, departing from cycle 0003's v003, which balanced both. Neither variant recovered the dual-improvement seen in cycle 0003, suggesting the hypothesis space may have been too focused on single levers or the magnitude of structural changes too coarse.

## Open questions

- Can v001's aggressive decay filtering be loosened (higher retention threshold, longer decay window) to preserve more ghost-queue candidates without losing the latency gains from reduced ghost-queue size?

- Is v003's latency overhead in occupancy tracking structural and unavoidable, or does scaling down the sizing adjustment range (e.g., ±5% instead of ±15%) proportionally reduce overhead while retaining hit-rate benefits?

- Could a hybrid approach—v003's dynamic sizing logic paired with v001's gentler frequency-decay ghost-queue management—recover the hit-rate gains without the latency penalty, or would the combined overhead exceed v003's individual cost?

## Continuation

The cohort confirms that **occupancy-responsive thresholds (from cycle 0003) remain the most effective lever**, but scaling occupancy-awareness to queue structure incurs non-trivial overhead. v001's ghost-queue frequency-decay approach is mechanically sound but over-aggressive; the next cycle should focus on parameter tuning rather than new mechanisms. Avoid further frequency-distribution experiments unless paired with occupancy as a secondary signal. **For the next cycle:** (1) tune v001's decay parameters (retention threshold, decay window) to preserve more ghost entries while maintaining latency gains; (2) test v003 with reduced sizing-adjustment magnitudes (±5–10%) to isolate whether overhead is proportional; (3) if dual-metric improvement is critical, explore hybrid configurations (v003's sizing + v001's tuned decay) rather than new signal types; (4) if latency stability is the priority, accept that v001's put-latency gains are available at hit-rate cost, but require hit-rate recovery through ghost-queue tuning, not structural sizing.
