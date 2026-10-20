# Variant v003 Insight

## Helped

- (none): hit-rate on workload-a is unchanged from the parent, so raising the initial access counter of new entries from 0 to 1 did not measurably change which entries the victim scan evicts on this trace.

## Hurt

- per-op-time (get): the tail of the get path moved slightly worse than the parent, with no change to get's median. The change only touches the insert-time counter write, so the tail shift is most likely noise rather than a mechanism; a repeat run would be needed to tell.
- per-op-time (put): the put median and tail moved marginally in the wrong direction. The initial counter is written on insert, which is a put, so this is the one path the change actually touches, but the shift is small enough to read as measurement noise.

## Takeaway

Seeding a fresh entry's counter at 1 instead of 0 is effectively inert under the parent's full-scan, lowest-counter eviction on workload-a: the hit rate is identical and metadata size is unchanged, so the parent's retention gain does not depend on how new entries enter the ranking. The useful signal is that the parent's benefit comes from per-hit counts accumulated after insertion, not from the insert-time offset, so further work on this counter should target how counts grow rather than their starting value.
