# Variant v002 Insight

## Helped

- hit-rate: The probation FIFO over slots `[0, small_len)` replaces a new item in place if it was never hit, so one-touch blocks now last only `small_len` misses. In v003 they lasted a full main-hand rotation. This gave the largest gain on web_0 at 131072, the cell the hypothesis predicted, and beat the refutation threshold there. The scan-heavy one-touch traffic no longer passes through the frequency-ranked main region, and it no longer forces the main sweep to decrement counters on reused blocks.
- hit-rate: hm_0 improved at both sizes. Two things combine here. Main-region victims are chosen only when a probation item earns promotion. And the GCLOCK sweep is restricted to `[small_len, len)`, so counters in main are decremented only on promotions, not on every miss. Reused blocks therefore keep their protection longer than under v003's whole-bucket sweep.

## Hurt

- hit-rate: web_0 at 16384 did not improve and was effectively flat, with a negligible dip. A small bucket gets a probation ring of only `max_size / 4` slots. That is too short a window for web_0 blocks whose second touch comes later than `small_len` misses. Those blocks are discarded as one-touch before they can be promoted, which cancels the scan-filtering gain at that size.

## Takeaway

Cold insertion does carry over to the GCLOCK lineage when it is built as an S3-FIFO-style probation ring in the same `Vec`. Gating admission to main on a hit during probation is what made GCLOCK scale at 1024-slot buckets, especially on web_0. Probation depth is the open knob. A fixed `len/4` ring is neutral in small buckets, where reuse distances are long relative to the ring size. So the next step is to tune or adapt `small_len`, and possibly to promote items with a non-zero counter instead of resetting it to 0. Per-op-time is still unmeasured. A promotion still costs at most one main sweep, the same bound as in v003.
