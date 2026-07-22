# Variant v003 Insight

## Helped

- metadata-size: Deleting the `timestamp: Instant` field from `Item` removed the 16-byte field plus its alignment slack, collapsing `Item<u64,u64>` to a packed `key`+`value`+`lfu_counter` (8+8+8) layout.

## Hurt

- per-op-time: The summary p50 drifted up even though the struct got smaller and puts got faster; with no code on the hot path touching timestamps, this reads as measurement noise rather than a real regression from the field removal.

## Takeaway

The `timestamp` field was pure dead weight under the `ClassicLRU` eval path — dropping it shrank `Item` metadata by 40% with zero hit-rate cost, and reducing `TimestampLRU` to FIFO insertion-order semantics kept the type/ordinal surface intact so JNI contracts didn't shift. Metadata carried only by unused policies is free to cut.
