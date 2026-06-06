# Variant v003 Insight

## Helped

- **hit-rate**: Increasing MAX_FREQ from 3 to 4 gives items one additional eviction round in the main queue, allowing moderately-reused items to prove their value through more recirculation attempts before permanent discard.

## Hurt

(none)

## Takeaway

Combining the 1/5 small-queue admission gate with deeper frequency tracking (MAX_FREQ=4) proves complementary: the tighter gate ensures only items with solid reuse signals enter the main queue, while the increased MAX_FREQ ceiling provides those selected items with more eviction-cycle opportunities to retain themselves. The small hit-rate improvement validates that structural selectivity and frequency-depth are orthogonal tuning axes that compound when applied together.
