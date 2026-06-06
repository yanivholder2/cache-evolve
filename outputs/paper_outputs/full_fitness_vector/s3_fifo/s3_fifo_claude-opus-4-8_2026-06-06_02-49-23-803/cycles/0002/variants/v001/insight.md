# Variant v001 Insight

## Helped

- hit-rate: tightening the admission split from `capacity / 10` to `capacity / 16` shifts another ~3.5% of capacity from the small queue into `main_size`, so the main resident queue holds more reuse-proven keys and survives more of workload-a's reaccesses. The narrower small queue still screens one-hit-wonders effectively at capacity 2048, so shrinking it costs no filtering quality.

## Hurt

- (none) metadata-size is untouched — the change only adjusts the integer `small_size`/`main_size` field values in `try_new`, not the `Entry`/`S3Fifo` struct layout. per-op-time did not regress meaningfully: total resident set still equals `capacity`, so `get`/`put` scan the same number of entries regardless of the split.

## Takeaway

On workload-a at capacity 2048 the small admission queue can shrink to ~6% (`capacity / 16`) and still screen one-hit-wonders, so the monotone sweep from v003's 10% split continues to pay off: every slot moved from the filter to the main queue raises hit-rate. The gain is small and shrinking, suggesting the split is approaching the point of diminishing returns where the filter starts to under-screen.
