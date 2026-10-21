# Variant v001 Insight

## Helped

- hit-rate: Cutting `ghost_cap` from `capacity - small_target` to `capacity / 2` (at size 2048, from 1843 remembered keys to 1024) means a key evicted from `small` must come back within a much shorter window of later small-queue evictions to count as a ghost hit. Keys that come back from loops longer than that window now re-enter `small` and must prove reuse in probation. Before, they went straight into `main`, where they displaced resident entries that were being reused.

## Hurt

(No dimension regressed. The fitness vector reports only `hit-rate`. The smaller ghost should also roughly halve the `ghost_q`/`ghost_set` footprint and its `with_capacity` reservations, but `metadata-size` and `per-op-time` were not measured, so that gain is unverified.)

## Takeaway

The ghost window size is a real tuning knob on workload-a, and the parent's near-capacity ghost was too loose. Admitting a ghost hit straight into `main` only helps when the window is short enough that a hit actually signals near-term reuse. Since shrinking the window helped and nothing else in the policy changed, the next probe on this axis should go smaller still (e.g. `capacity / 4`) to find where too few genuine re-references are caught. Fingerprinting or other structural ghost changes should wait until the size optimum is found.
