# Variant v001 Insight

## Helped

- hit-rate: Shrinking the admission queue further from `capacity/10` to `capacity/16` (~6%) handed even more slots to the main resident queue, letting hot keys survive additional second-chance passes and edging the measured hit-rate above the v002 parent.

## Hurt

## Takeaway

Pushing the small/main split one step past v002's 10/90 toward ~6/94 still pays off on this workload — a 6%-capacity admission queue continues to filter one-hit-wonders into the ghost queue without yet leaking them, so the enlarged main queue is the source of the gain. The improvement is small and shrinking relative to v002's step, signaling the partition ratio is approaching its plateau and further shrinking the filter will soon start to hurt rather than help.
