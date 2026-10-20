# Variant v003 Insight

## Helped

- hit-rate: shrinking the small admission queue from 25% to ~10% of capacity (`capacity / 10`) handed the freed slots to `main_size`, so the main resident queue holds more reuse-proven entries and retains them across more of workload-a's reaccesses.

## Hurt

- (none) metadata-size is untouched — the struct layout and `Entry`/`S3Fifo` byte sizes are independent of the small/main split, which only changes integer field values, not memory footprint.

## Takeaway

Devoting capacity to the main queue rather than the admission filter lifts hit-rate on workload-a: the small queue only needs to be wide enough to screen one-hit-wonders, and at this capacity (2048) a ~10% filter is sufficient while leaving more room for frequently reused keys. The large per-op-time drop is not a structural consequence of this change — `get`/`put` still scan the full `small`+`main` resident set whose total equals `capacity` regardless of the split — so it most plausibly reflects measurement variance rather than the bucket resizing itself.
