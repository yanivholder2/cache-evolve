# Variant v001 Insight

Note: I could not list or diff the overlay source in this session (only Read and Write were usable). This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- hit-rate: A larger two-hash doorkeeper with aging reset gives fewer false positives and collisions than the small lossy filter of cycle 0006. Fewer cold keys slip in, and fewer first-sighting keys are wrongly rejected. Hit-rate rose above the parent's.

## Hurt

- per-op-time: Both get and put got slower, including the put tail. The put path now does two hash-bit probes into a much bigger bit array and occasionally runs a popcount-and-clear reset under the lock. A larger array also adds cache misses. The get path was not meant to change, so its slowdown likely comes from cache pressure or noise.

## Takeaway

Enlarging and aging the doorkeeper recovered the hit-rate that the small filter lost. It did so at the cost of put-side latency, which was the dimension the hypothesis meant to improve. Admission accuracy and put cost trade off directly, so a better filter does not speed up the put path. Any later speedup must come from making the check cheaper, for example by keeping it in a single cache line, and not from filtering more keys.
