# Variant v001 Insight

## Helped

- metadata-size: No change; the tags apparently fit without growing the per-item or cache struct, so the constraint in the hypothesis was met (but nothing improved).

## Hurt

- per-op-time (get): The tag-first scan was meant to cut memory touched per get, but get got slower, including at the tail. The tag check is an extra step on top of a bucket that is probably already small and cache-resident, so it adds work without saving memory traffic.
- per-op-time (put): Put slowed much more than get. Every insert, swap, and promote must now move tags in lockstep with entries, which roughly doubles the shifting work on the write path.
- hit-rate: The hypothesis required identical hit behavior, but hit rate dipped slightly. This suggests the tag bookkeeping drifted from the parent's victim selection or ordering, for example through a tag collision or a slot that wasn't swapped. I could not inspect the source diff, so this is inferred.

## Takeaway

Adding a parallel tag array to a small, already-compact bucket did not make lookups cheaper. It made every mutation costlier and slightly changed behavior. Memory layout is not the bottleneck for get at this bucket size, and any layout change that must be kept in sync across promotions and swaps risks both write cost and policy fidelity.
