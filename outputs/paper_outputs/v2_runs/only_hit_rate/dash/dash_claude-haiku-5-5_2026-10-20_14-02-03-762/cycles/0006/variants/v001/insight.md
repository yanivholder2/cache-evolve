# Variant v001 Insight

## Helped

- hit-rate: The hypothesis replaced the one-bit seen-in-stash marker with a 2-bit saturating reuse counter and promoted stash items only at counter >= 2. The gain over the parent is marginal on LIRS workload-a at size 2048, so this is a weak signal rather than a clear win. One trace and one run cannot separate a real effect from noise.

## Hurt

## Takeaway

Gating stash promotion on a saturating reuse count, instead of a single re-hit bit, produced at most a marginal hit-rate change on workload-a at size 2048, so the hypothesis that persistent reuse separates from a single revisit is not supported strongly by this trace. The overlay diff was not inspected in this pass (the directory listing was unavailable), so I could not confirm the counter's packing into the entry metadata, the reset on demotion or eviction, or that the 0.75 occupancy branch behaves as specified. This result describes the intended change, not a verified implementation.
