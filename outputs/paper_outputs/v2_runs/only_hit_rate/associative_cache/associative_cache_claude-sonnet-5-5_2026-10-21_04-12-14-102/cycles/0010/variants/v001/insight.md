# Variant v001 Insight

## Helped

- (none): the only dimension, hit-rate, did not improve over the parent.

## Hurt

- hit-rate: Doubling the probation segment from slots/16 to slots/8 shrinks the protected segment. Re-referenced lines get less protected room, and the skip-move shortcut applies to a smaller protected top quarter. Both cost a little on the lirs workload-a trace at size 2048.

## Takeaway

With the 4x-wide skip-move split left unchanged, the larger slots/8 probation does not raise hit rate. It is slightly worse than slots/16, so extra probation room trades away protected capacity for no gain. The effect is small, which suggests probation size is a weak lever here. Keep slots/16 and look at other levers, such as insertion point or demotion.
