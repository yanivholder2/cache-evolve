# Variant v003 Insight

## Helped

- hit-rate: Hit rate rose slightly over the baseline. The flat `Vec<u8>` sketch with single-hash bit-slice indexing and conservative update kept frequency estimates good enough for admission, and the small gain is plausibly within noise on one trace and one cache size.

## Hurt

- metadata-size: The vector has no entry for this dimension, so the claimed reduction in sketch footprint was never measured. The hypothesis's main goal is therefore unverified.
- per-op-time: This dimension is also missing, so the claimed speedup from one hash and contiguous memory is unverified.

## Takeaway

Repacking the sketch into a flat u8 array with single-hash indexing did not hurt admission quality, but this evaluation only measured hit-rate. It shows nothing about the size and latency benefits the change was meant to deliver. Before treating the packing as a win, the metadata-size and per-op-time measurements need to run for this variant. A hit-rate difference this small on one trace should not count as evidence either way.
