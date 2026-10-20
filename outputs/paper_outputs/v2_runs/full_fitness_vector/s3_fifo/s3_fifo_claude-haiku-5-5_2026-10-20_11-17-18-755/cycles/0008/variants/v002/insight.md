# Variant v002 Insight

## Helped

- metadata-size: The container-level `S3Fifo` footprint shrank, which fits the hypothesis's removal of SipHash's keyed state from the resident index map, while the per-entry `Entry` size stayed the same as the parent.
- per-op-time: Put latency at the median dropped slightly, consistent with the cheaper multiply-shift hash on the put path, although the put tail widened.

## Hurt

- per-op-time: The get path got slower at both the median and the tail, the opposite of the hypothesis's prediction, so the fixed multiply-shift hasher did not reduce the cost of the single index lookup on a hit.

## Takeaway

Swapping SipHash for a fixed multiply-shift hasher on the `u64` index shrank the container and left hit rate untouched, as expected, but did not produce the get-latency gain the hypothesis predicted. The get regression is most likely timing noise or a layout effect, since cycle 0007 already showed shared noise on unchanged get paths, so the get-path baseline should be rerun before crediting or discarding this change.
