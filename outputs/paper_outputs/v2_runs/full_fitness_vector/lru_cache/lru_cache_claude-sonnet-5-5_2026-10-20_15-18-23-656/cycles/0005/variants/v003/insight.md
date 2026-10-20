# Variant v003 Insight

## Helped

- per-op-time: get and put latencies dropped even though the hypothesis changed only a construction-time constant. The hot path is untouched, so this is most likely measurement noise or layout and cache effects, not a real gain from the ratio change.

## Hurt

- hit-rate: Growing the protected segment to 3/4 of capacity shrinks the probationary segment that absorbs one-shot scan entries. Scan traffic therefore evicts more of the entries that would have been re-referenced. The hit-rate drop is small.

## Takeaway

Moving the protected/probationary split to 3/4 protected did not improve hit-rate on the scan-heavy LIRS trace and slightly lowered it. The ratio is a weak lever here, and a larger probationary share looks at least as good. Metadata-size was unchanged, as expected for a constant-only edit. The per-op-time shift should not be credited to this change.
