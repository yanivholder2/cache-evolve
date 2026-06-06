# Variant v001 Insight

## Helped

- (none observed): The pre-increment `PROMOTE_FLOOR = 2` gate now *can* fire — threading `prior` (captured before `sketch.increment`) into `promote` fixes cycle 0006's inert always-true floor — but it did not move hit-rate on workload-a at size 2048.

## Hurt

- (none observed): hit-rate is the only measured dimension and it held exactly at the parent's level; keeping a first-touch node probationary instead of promoting it for free neither cost nor gained a hit here.

## Takeaway

Correctly evaluating the entry floor pre-increment makes it engage mechanically, but it changes outcomes only during the brief spare-room warm-up window — and on this trace the probationary keys that earn a hit during that window have already been seen at least twice (`prior_freq >= 2`), so the floor admits exactly the keys the unconditional parent already admitted. The lever is now live but finds nothing to gate: once protected fills, the unchanged frequency contest dominates, leaving hit-rate untouched.
