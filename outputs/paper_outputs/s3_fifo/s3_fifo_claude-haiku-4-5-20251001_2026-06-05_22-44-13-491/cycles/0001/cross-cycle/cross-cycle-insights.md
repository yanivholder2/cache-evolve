# Cross-Cycle Insights

## Recurring wins

- **Conservative, single-dimension tuning** beats aggressive multi-factor rewrites. Modest parameter adjustments (e.g., frequency depth +1) that respect existing calibration outperform blanket policy shifts.
- **Alignment with workload reuse patterns** is more valuable than theoretical mechanisms. Success comes from matching tuning to observed access behavior, not from logically sound queue mechanics.

## Recurring failures

- **Upfront protection without earning thresholds** crowds out high-frequency items. Ghost-hit promotion that grants immediate frequency boosts penalizes systems where protection must be justified by evidence.
- **Static congestion thresholds** appear misaligned with real queue dynamics. Fixed thresholds (e.g., 80% load) do not reliably trigger adaptive behavior in practice.

## Open questions

- Can frequency depth (MAX_FREQ) be predicted from workload reuse-depth distributions, or must it be discovered empirically per workload?
- Does ghost-hit protection benefit from being *earned* (tracked re-admission) rather than *granted* upfront?
- At what point does increasing frequency depth hit diminishing returns, and does this vary predictably by workload?

## Per-policy notes

- **S3-FIFO**: Frequency balance is tightly calibrated. The system is sensitive to frequency-depth changes but robust to modest adjustments aligned with reuse patterns. Dynamic congestion-responsive strategies require careful thresholding to match actual queue behavior; static trigger points may be insufficient.
