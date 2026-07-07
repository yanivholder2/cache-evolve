# Variant v003 Insight

## Helped

- hit-rate: Widening buckets from 128×16 to 32×64 and switching to pure min-counter LFU (counter capped at 4, no positional movement) collectively lifted hit-rate substantially over the baseline's positional-LRU on narrow buckets.

## Hurt

## Takeaway

Pure min-counter LFU on wide buckets closes most of the gap to the best-known accumulated-mechanism score but does not match it: the simpler two-change stack (wide geometry + frequency-only eviction) approaches but falls short of the configuration that also incorporates cold-first bias, mid-point insertion, and decay. This tells us the auxiliary mechanisms in the prior best-known config contribute a small but real and non-redundant lift — frequency signal alone with a wide candidate pool is nearly sufficient, but stripping all positional and decay structure costs a measurable margin. The counter cap at 4 is not enough on its own to replicate what cold-first insertion and decay do for cold-supply replenishment.
