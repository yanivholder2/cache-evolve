# Variant v002 Insight

## Helped

- hit-rate: The doorkeeper keeps first-sighting keys out of the counters, so one-hit scan keys no longer inflate sketch frequencies. Reused keys are then separated slightly better from scan keys at the admission gate, and the effect is a very small gain over the parent.

## Hurt

- hit-rate: A first-time key now estimates only 1, so a genuinely new hot key is more likely to lose the gate comparison against a probation victim. Clearing the doorkeeper at every halving also forgets recent first sightings. These effects offset most of the pollution reduction.

## Takeaway

Putting a one-bit doorkeeper in front of the sketch barely changed hit-rate. The parent's segmented LRU with a 95% protected share and the sketch gate had already removed most of the scan pollution, so the sketch had little saturation left to fix. Gains probably need a different lever than a better-filtered frequency estimate.
