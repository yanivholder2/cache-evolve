# Variant v003 Insight

## Helped

- (none observable): the only measured dimension is hit-rate, and a pure
  link-encoding swap cannot move hit-rate by construction.

## Hurt

- (none observable): the repacking is bit-for-bit behaviour-preserving, so
  no measured dimension regresses.

## Takeaway

H03 is a representation-only change — `Option<usize>` link fields collapse
to `u32` with a `NIL = u32::MAX` sentinel, and `try_new` adds a
`capacity < NIL` guard so a real arena slot can never collide with the
sentinel. The intended win is on metadata size (per-node link overhead
drops from 32 bytes to 8 on a 64-bit target), but the fitness vector here
reports only hit-rate, so the payoff is invisible: this variant's apparent
hit-rate movement against the baseline reflects different trace/size
inputs, not the encoding change. The lesson is structural — until
metadata-size is actually scored, niche-style packing wins like this can't
be selected for and look like neutral noise.
