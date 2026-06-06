# Variant v002 Insight

## Helped

- per-op-time (GET p50): Lower median latency suggests the absolute frequency floor reduces admission conflicts, leading to faster average-case lookups even though longer-tail behavior degrades.
- per-op-time (PUT p50): Median put operations improve by filtering out low-frequency items earlier, reducing the number of candidates for eviction consideration.
- per-op-time (PUT p99): Tail latency on puts also improves, indicating more predictable rejection patterns for items below the frequency floor.

## Hurt

- per-op-time (GET p99): Significant tail-latency spike on get operations suggests the frequency floor creates pathological access patterns where occasional low-frequency key lookups trigger expensive sketch operations or cache-miss chains.
- hit-rate: No improvement despite the hypothesis's intended benefit; the filtering does not enhance selectivity for this workload, suggesting the baseline's relative-only admission gate was already near-optimal or the noise floor on this trace is higher than frequency=1.

## Takeaway

Adding an absolute frequency floor (`incoming_freq >= 2`) reduces average put latency and median get latency by filtering out low-frequency admissions earlier, but it does not improve cache hit-rate and introduces a severe tail-latency regression on get operations. The mechanism works as intended—fewer items are admitted—but the workload does not benefit; the stricter gate either rejects future-hot items or creates unpredictable sketch behavior on misses. Relative-only admission gating remains superior for this trace.
