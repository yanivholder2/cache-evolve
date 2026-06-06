# Variant v003 Insight

## Hurt

- hit-rate: Relative frequency normalization changed the victim selection formula without providing benefit; the additional complexity of dividing age by (1 + freq/max_freq) appears to introduce noise rather than sharpen eviction decisions compared to the simpler frequency-only admission gate.

## Takeaway

Normalizing recency by relative frequency depth did not improve discrimination of cold items. The baseline's frequency sketch combined with simple frequency-based admission already captures the essential signal; the added complexity of age-weighted scoring normalized to the hottest item in cache overcomplicates victim selection and degrades the working-set protection property TinyLFU relies on.
