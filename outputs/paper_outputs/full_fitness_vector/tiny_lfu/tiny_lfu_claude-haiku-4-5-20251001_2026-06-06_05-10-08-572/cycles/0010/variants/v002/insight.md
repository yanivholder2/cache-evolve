# Variant v002 Insight

## Helped

- put p50: The additional candidate scan appears to not measurably impact median put latency, suggesting the overhead remains within noise margins even with N=4 evaluation.

## Hurt

- hit-rate: Extending the victim pool to N=4 yields only a 0.03% gain versus the 0.37% improvement at N=3, indicating sharply diminishing returns as victim candidate count increases beyond the N=3 sweet spot.
- get p99: Evaluating a 4th candidate extends the traversal path during eviction-triggered frequency lookups, increasing worst-case get latency by ~25ns, showing the cost of deeper scanning now impacts read tail performance.

## Takeaway

N=4 represents a boundary where diminishing returns become acute. The hit-rate gain collapses to 0.03% compared to N=3's 0.37%, while get tail latency deteriorates noticeably. This confirms that N=3 occupies the true sweet spot: it captures meaningful frequency-diversity benefits without the latency pathology that emerges as scanning deepens. The result validates the hypothesis's premise that a checkpoint between proven N=3 and pathological N=5 exists, and that checkpoint lies at or before N=4.
