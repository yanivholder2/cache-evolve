# Variant v003 Insight

## Helped

- hit-rate: Hypothesis H03 narrowed the frequency-sketch counters to 8-bit saturating values and spent the freed bytes on roughly eight times as many columns per row, so collisions in the count-min sketch should drop. The hit-rate on workload-a at size 2048 moved up slightly relative to baseline, which is consistent with lower collision noise in the admission comparison.
- per-op-time: Both get and put latencies dropped against baseline at p50 and p99. The hypothesis made no timing claim, and the only per-operation change it describes is the column-index modulus, so the gain is not clearly explained by the stated change and may be partly noise. Treat it as unconfirmed.

## Hurt

- None. The metadata-size dimension is unchanged, consistent with the hypothesis keeping the sketch within the baseline byte budget.

## Takeaway

Keeping the sketch byte budget fixed while trading counter width for column count produced a small hit-rate gain and no metadata regression, so the collision-reduction direction looks worth pursuing. The source diff could not be inspected in this session, so the row and column counts the hypothesis asked to record are not confirmed here, and the saturation-at-15 cost to admission resolution remains untested in isolation.
