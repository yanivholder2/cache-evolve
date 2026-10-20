# Variant v002 Insight

## Helped

- None. Hit-rate on lirs workload-a at capacity 2048 is unchanged from the baseline, so the eager-promotion hypothesis did not produce the predicted gain.

## Hurt

- None measured. Hit-rate is flat, and this variant's fitness vector reports no metadata-size or per-op-time entries, so no regression on those dimensions can be attributed to the change.

## Takeaway

Moving a small-queue entry to the front of main on its second hit, rather than at small-queue eviction, does not change hit-rate on this workload at this capacity. The baseline already promotes any small entry with freq > 1 when it reaches the tail, so the change mostly shifts the timing of that promotion without changing which keys stay resident. Any future promotion-timing lever needs to alter the eviction decision itself, not just when an already-warm key moves queues.
