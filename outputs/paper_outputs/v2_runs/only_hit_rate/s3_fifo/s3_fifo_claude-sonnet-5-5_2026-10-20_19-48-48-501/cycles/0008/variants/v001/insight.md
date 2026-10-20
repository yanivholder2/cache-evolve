# Variant v001 Insight

## Helped

- (none): hit-rate did not improve; the ghost-hit feedback loop did not find a small-queue size better than the fixed ~5% ratio.

## Hurt

- hit-rate: it regressed slightly versus the parent. Resizing `small_target` every `capacity` requests on ghost-hit and promotion counts likely moved the small queue away from a size that was already near-optimal for this trace. The window-edge thresholds (2% ghost hits to grow, 0% ghost hits plus under 0.5% promotions to shrink) may also have fired on noise.

## Takeaway

Adaptive small-queue sizing from ghost-hit and promotion rates did not beat the fixed ~5% ratio on this workload. The fixed ratio looks like a real local optimum rather than a hand-tuning gap, and the feedback signal was too coarse or noisy to improve on it. Note: I could only use Read/Write in this session and did not diff the overlay sources. This interpretation rests on the hypothesis and the fitness vectors, not on the code.
