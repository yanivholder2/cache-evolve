# Variant v001 Insight

## Helped

- per-op-time: Get p50 latency fell, most likely because periodically-reset counters stay at smaller magnitudes and generate simpler branch paths in `increment`; note that put p99 rose sharply on the turns when the full-sketch reset sweep fires.

## Hurt

- hit-rate: The `width / 2` reset threshold fires too frequently relative to the working-set size, collapsing all frequency estimates toward zero after each sweep and stripping the sketch of its ability to distinguish long-lived hot residents from cold newcomers — the admission filter's selectivity degrades rather than adapting to phase shifts.

## Takeaway

Counter decay at the Caffeine-style `width / 2` cadence backfires on workload-a: the LIRS trace's working set is stable enough that long-term frequency accumulation is actually a reliable admission signal, and aggressive halving destroys that signal faster than new accesses can rebuild it. Any decay strategy for this workload needs a much longer reset interval — or a dynamic threshold tied to observed access entropy — before it can outperform a plain accumulating sketch.
