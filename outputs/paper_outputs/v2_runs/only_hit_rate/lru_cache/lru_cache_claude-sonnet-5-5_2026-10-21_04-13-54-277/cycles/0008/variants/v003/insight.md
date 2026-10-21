# Variant v003 Insight

## Helped

- hit-rate: Conservative-update increments (only the minimum-valued counters are bumped) reduce collision inflation in the count-min sketch. The admission comparison therefore sees slightly cleaner frequency estimates, and the hit rate improved marginally over the parent.

## Hurt

- (none observed in the measured dimensions)

## Takeaway

Conservative updates with saturating 4-bit counters did not cost hit rate, and the gain was small. The hypothesis's main target, metadata-size, is not in this variant's fitness vector, so the intended halving of sketch bytes is unverified here. Sketch precision looks like a minor lever on workload-a at this cache size. I could not read the overlay source in this session, so this interpretation rests on the hypothesis text and the fitness vectors alone.
