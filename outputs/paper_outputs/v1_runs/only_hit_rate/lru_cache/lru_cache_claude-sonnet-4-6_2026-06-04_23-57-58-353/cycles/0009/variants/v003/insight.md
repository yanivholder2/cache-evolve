# Variant v003 Insight

## Helped

- hit-rate: The staleness multiplier (`irr * (op_clock - last_access)`) correctly reclassified entries that had a tight historical burst but had gone untouched for many operations, allowing the scanner to prefer evicting them over entries with a moderate but more recent access gap.

## Hurt

_(none — only one fitness dimension is tracked)_

## Takeaway

Replacing pure IRR with the staleness-weighted score `irr * staleness` produces a small but real hit-rate gain, confirming that long-silent tight-burst entries are genuinely poor candidates for retention and that pure IRR was retaining some of them incorrectly. The gain is modest rather than large, which indicates that on this LIRS-structured trace the majority of eviction decisions were already correct under pure IRR — the staleness correction rescues only a narrow slice of borderline cases where an entry's raw IRR happened to look hot while its actual last access was far in the past.
