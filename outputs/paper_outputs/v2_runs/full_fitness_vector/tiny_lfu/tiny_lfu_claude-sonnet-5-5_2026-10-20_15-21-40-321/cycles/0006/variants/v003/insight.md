# Variant v003 Insight

(Note: the overlay source could not be read in this session; this interpretation rests on the hypothesis and the two fitness vectors.)

## Helped

- hit-rate: no movement. Conservative update was meant to reduce collision inflation, but admission decisions ended up the same as the parent's.

## Hurt

- per-op-time: get p50 and put p99 moved only slightly, and put p50 moved slightly the other way. The extra read-then-conditional-increment pass in the sketch `increment` adds a little work and branching, but the shifts are small enough to be mostly noise.

## Takeaway

Switching the count-min sketch to conservative update changed nothing the workload could see. With the sketch at cap=1024 and strict `>` admission, collision inflation is evidently not what limits hit-rate on this trace. Metadata size is unchanged, since the change adds no state. Further gains probably need to come from the admission policy, the aging policy or a window segment, not from making the frequency estimates more precise.
