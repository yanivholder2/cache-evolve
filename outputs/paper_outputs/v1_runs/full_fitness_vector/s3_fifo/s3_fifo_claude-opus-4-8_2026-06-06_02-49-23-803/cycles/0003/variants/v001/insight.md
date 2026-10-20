# Variant v001 Insight

## Helped

- hit-rate: halving the small-queue divisor from `capacity / 16` to `capacity / 32` moves another ~3% of capacity into `main_size`, so the reuse-proven main queue holds more of workload-a's reaccessed keys; at capacity 2048 the ~64-slot small filter still screens one-hit-wonders, so the shift was pure gain.

## Hurt

- (none) metadata-size is untouched — only the integer `small_size`/`main_size` values in `try_new` changed, not the `Entry`/`S3Fifo` struct layout. per-op-time did not regress: the resident set still totals `capacity`, so `get`/`put` traverse the same number of entries regardless of the split.

## Takeaway

On workload-a at capacity 2048 the split knee has not yet been reached — shrinking the small admission filter to ~3% (`capacity / 32`) still screens one-hit-wonders and the freed slots in the main queue keep raising hit-rate. The gain is smaller than the prior `/16` step, confirming the sweep is monotone but flattening, so the under-screening boundary is likely just past `/32`.
