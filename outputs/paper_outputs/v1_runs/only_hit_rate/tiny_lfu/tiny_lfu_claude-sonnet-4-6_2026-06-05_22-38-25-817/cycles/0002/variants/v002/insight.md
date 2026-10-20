# Variant v002 Insight

## Helped

*(none)*

## Hurt

- hit-rate: The `capacity × 100` reset cadence still causes a marginal regression versus the no-aging baseline, because halving all sketch counters once per full-population turnover erodes just enough frequency separation to cause the admission gate to make slightly worse admit/reject decisions on workload-a.

## Takeaway

Even at 10× the cadence of v001's already-failing `capacity × 10` schedule, periodic sketch aging continues to slightly hurt hit rate on workload-a rather than help. This suggests that on this trace the hot-key frequency distribution is stable across the entire replay window: the baseline's ever-accumulating counts maintain clean separation between hot and cold items throughout, and any half-life reset — however infrequent — discards signal that the admission gate was relying on. Sketch aging appears to be a net positive only when frequency distributions genuinely shift over time and stale counts become noise; workload-a does not exhibit that property at this cache size.
