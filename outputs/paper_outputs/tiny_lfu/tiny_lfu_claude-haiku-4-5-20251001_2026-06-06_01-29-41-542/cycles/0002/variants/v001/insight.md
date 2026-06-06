# Variant v001 Insight

## Helped

- hit-rate: Doubling sketch width from 8x to 16x provides further collision reduction, enabling marginally more accurate frequency discrimination in the admission gate.

## Hurt

(none)

## Takeaway

Widening the Count-Min sketch from 8x to 16x capacity yielded only a +0.0006 hit-rate improvement, demonstrating strong diminishing returns. While the previous cycle's 4x→8x jump delivered +0.0047 gain, this step confirms that sketch width's leverage on admission accuracy plateaus as relative width increases. The collision-reduction benefit is real but shrinking; further width increases are unlikely to justify their memory cost relative to other tuning vectors.
