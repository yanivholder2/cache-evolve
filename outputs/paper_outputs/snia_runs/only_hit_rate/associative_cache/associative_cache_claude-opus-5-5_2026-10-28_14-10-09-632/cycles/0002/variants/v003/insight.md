# Variant v003 Insight

## Helped

- hit-rate: Changing `Item::lfu_counter` from a 1-bit CLOCK flag to a GCLOCK counter that saturates at `GCLOCK_MAX = 3` raised hit-rate on all four trace/size cells. The largest gain was hm_0 at 131072. In a large bucket, a sweep clears many items' state at once, and under the 1-bit flag a frequently hit block lost all its protection in one pass. Under the counter it loses only one level, so it survives up to three more sweeps while new blocks, which start at 0, are evicted first.
- hit-rate: The gains on web_0 were smaller but still positive, at both sizes. This includes web_0 at 131072, the one cell where the 1-bit parent had slipped. So the extra frequency levels fixed the FIFO-like drift that the 1-bit sweep showed in a large bucket where most bits were set.

## Hurt

(No dimension regressed. Hit-rate is the only dimension in the fitness vector.)

## Takeaway

A 2-bit saturating counter in the field that already exists is enough to make in-place CLOCK frequency-aware and resistant to scans. It improves on the 1-bit parent everywhere, and it adds no per-item state and no element shifting. The benefit grows with bucket size and with how much reuse the trace has (hm_0 at the large size). That fits the mechanism: the counter matters most when a single sweep under the 1-bit flag would erase protection from many reused blocks at once. The cost side is still unmeasured. A full-bucket miss can now sweep up to about three rotations instead of one, and `eviction_candidate_index` scans the whole bucket when no counter is 0. Per-op-time needs to be benchmarked before this can be called a strict win over v001-style ordering.
