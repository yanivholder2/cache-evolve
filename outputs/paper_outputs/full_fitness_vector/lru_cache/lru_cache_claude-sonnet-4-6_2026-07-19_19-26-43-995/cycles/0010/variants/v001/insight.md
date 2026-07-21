# Variant v001 Insight

## Helped

- **metadata-size**: Replacing `RandomState` with the inline zero-sized `FnvBuildHasher` removed the two `u64` seed fields that `RandomState` stores, shrinking `LruCache<u64,u64>` by 16 bytes — though the tracked summary anchors on `Node<u64,u64>` (which was already at its minimum from the prior link-separation change), so this gain does not appear in the headline score.

## Hurt

_(no tracked dimension summary worsened)_

## Takeaway

Substituting FNV-1a for SipHash delivered exactly the predicted 16-byte reduction in per-cache-instance struct size, validating that `FnvBuildHasher`'s zero-size makes a measurable structural difference. However, FNV-1a produced no detectable speedup in per-op latency at the benchmark's timer resolution, and hit-rate was unaffected as expected. The remaining open question is whether the `metadata-size` dimension's summary should track the cache-struct size rather than (or in addition to) the node size, since the actual allocation-time win is real but currently invisible to the fitness scorer.
