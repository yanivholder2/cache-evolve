# Variant v002 Insight

## Helped

- (none) The freq=1 ghost seed did not improve any dimension; hit-rate, the
  target axis, regressed against the parent.

## Hurt

- hit-rate: seeding re-admitted ghost keys with `freq = 1` instead of `0`
  gave each one a guaranteed survival of the `evict_main()` second-chance
  decrement, so proven-but-cold keys lingered in main an extra pass and
  displaced entries with live reuse, lowering hit-rate below the parent.
- per-op-time: put p99 rose because the longer-surviving freq=1 entries kept
  main fuller of low-value keys, making each `evict_main()` walk more
  second-chance decrements before finding a victim.

## Takeaway

A ghost hit proves *past* near-reuse, not that the key deserves the same
durability as a graduated small-queue entry — granting it a freed
second-chance pass over-protects keys whose reuse already happened, crowding
main and costing hit-rate. The graduation-vs-readmission asymmetry the
hypothesis tried to "fix" is load-bearing: re-admitted ghost keys should
re-prove themselves from freq=0, not inherit survival credit.
