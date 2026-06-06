# Variant v002 Insight

## Helped

- (none observed): seating a proven-hot newcomer straight into protected on the full-cache admission branch did not lift hit-rate; the case it targets (a churned-out hot key rescued before re-eviction) was already being caught well enough by the existing probation-then-promote path.

## Hurt

- hit-rate: the direct-to-protected admission slipped a hair below the parent — newcomers admitted on a single high sketch estimate skip probation's "earn a real hit" proof, so keys whose estimate is inflated by collisions or a one-off burst take protected room without demonstrating genuine reuse, and the spare-room guard mostly fires during fill/post-demotion gaps where that misplacement costs more than it saves.

## Takeaway

Steering placement from the *admission* side — letting a newcomer bypass probation into protected when its sketch estimate beats the protected head — is net slightly negative on workload-a at size 2048: the sketch estimate alone is a weaker admission signal than an observed in-cache hit, so skipping the probation filter admits the occasional false-hot key. The probationary tail's earn-a-hit gate is doing useful work, and removing it for "hot" arrivals trades it away for nothing measurable here.
