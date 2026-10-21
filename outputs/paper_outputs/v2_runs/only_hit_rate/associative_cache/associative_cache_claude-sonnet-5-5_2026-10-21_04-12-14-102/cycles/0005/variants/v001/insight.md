# Variant v001 Insight

## Helped

- hit-rate: No improvement. The only dimension is hit-rate and it did not rise over the parent.

## Hurt

- hit-rate: Fixing probation at a constant 2 slots, instead of slots/16 (min 1), made probation too small. Newly inserted entries were evicted before a second touch could promote them. That cost a little hit-rate on the LIRS trace relative to v003.

## Takeaway

The downward trend from shrinking probation has turned. Going below the slots/16 sizing slightly hurts, so v003's probation size is at or near the optimum for this trace. Further gains will need a different mechanism, such as promotion policy or protected-segment handling, not a smaller probation segment.
