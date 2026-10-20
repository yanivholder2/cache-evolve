# Variant v001 Insight

## Hurt

- hit-rate: Removing frequency weighting from secondary ranking within the cold cohort increased eviction of items that retained modest frequency signal, causing the pure-age approach to be overly aggressive in selecting victims.

## Takeaway

Pure age-based victim selection within the frequency-cold cohort reduces hit-rate relative to the hybrid age/(freq+2) formula. Even among low-frequency items, the frequency denominator provides meaningful signal to distinguish between genuinely dormant items and those with occasional recurrence; stripping it away causes the eviction policy to discard items that retain marginal value, worsening overall cache performance.
