# Variant v003 Insight

## Helped

(none — no fitness dimension improved)

## Hurt

- hit-rate: Doubling `bucket_size` and halving both bucket counts spread the
  same slots across fewer, wider buckets; the deeper per-bucket LRU stack did
  not offset the coarser hashing, so hot keys collide into fewer landing spots
  and the skewed trace's hit-rate dipped below baseline.

## Takeaway

On this workload, trading bucket count for bucket width is net-negative: deeper
LRU stacks do not compensate for the loss of hash-bucket addressability, so hot
keys contend more rather than survive longer. Width is not a free associativity
knob here — at conserved budget, having more buckets matters more than having
deeper ones, so future variants should keep bucket counts high rather than
widen them.
