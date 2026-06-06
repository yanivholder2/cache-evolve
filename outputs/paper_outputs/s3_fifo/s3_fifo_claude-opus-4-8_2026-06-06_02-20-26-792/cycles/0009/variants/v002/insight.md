# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Gating the split behind a `steer_accum >= STEER_THRESHOLD` hysteresis band (needing 4 net agreeing ghost re-admissions before moving `small_size` by one) made the controller too sluggish — it slowed the partition's drift toward main without sharpening it, so it captured slightly fewer of the hits v001's per-event nudging already reached on workload-a.

## Takeaway

On workload-a the per-event steering noise v001 suffered was not the bottleneck: the suspected "thrash" was largely real, directional signal, so requiring a 4-event majority before committing a move mostly just added lag rather than denoising. Hysteresis is the right discipline for genuinely mixed bursts, but here the threshold throttled a controller that was already pointed the right way — implying a lower threshold (or none) tracks this trace's slow, monotone imbalance better than a wide accumulation band.
