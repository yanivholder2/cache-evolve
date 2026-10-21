# Variant v001 Insight

## Helped

- hit-rate: The hypothesis added a probationary region to each 64-way bucket. New entries are admitted there and promoted only on a second hit, and eviction takes probationary entries first. One-touch entries from scans in the LIRS trace can no longer push out re-referenced protected entries. This gave a clearly larger gain over the parent than the parent's own change (widening associativity) did.

## Hurt

- No dimension regressed. Only hit-rate was measured, so any cost from the extra per-bucket partition bookkeeping (probation counter or flag, and a more involved eviction path) is unobserved, as with the parent.

## Takeaway

On this LIRS trace, the admission and eviction policy limits hit-rate more than geometry does. Separating probationary from protected entries beat the parent's geometry-only change without touching capacity or the public API. The next steps are to tune the probation size (about 16 of 64 slots here) and to start measuring per-op-time and metadata-size, which are still untested.
