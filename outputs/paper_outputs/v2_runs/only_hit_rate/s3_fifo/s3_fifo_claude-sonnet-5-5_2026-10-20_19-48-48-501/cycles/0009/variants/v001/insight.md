# Variant v001 Insight

## Helped

- hit-rate: no movement versus the parent. The hit-rate summary and the per-trace value on workload-a at size 2048 match the parent's.

## Hurt

- (none) No dimension regressed. The only measured dimension, hit-rate, is unchanged.

## Takeaway

Moving an entry from small to main on a hit with freq >= 2 gave no measurable gain on top of the parent's sizing (small queue of about 5%, shortened ghost) and raised freq cap. The most likely explanation is that the eviction-time promotion already moves those entries into main before they are evicted, so promoting earlier on the hit path changes little on this trace. Any gain from the parent-to-child lineage seems to come from the sizing and cap changes, not from the promotion timing. This is inferred from the identical results, because I could not read the overlay source to confirm the diff.
