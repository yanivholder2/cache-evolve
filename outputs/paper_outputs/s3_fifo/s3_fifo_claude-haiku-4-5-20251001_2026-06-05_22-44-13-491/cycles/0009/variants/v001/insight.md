# Variant v001 Insight

## Helped

- hit-rate: Reducing the small queue to 1/15 of capacity (from 1/14) further restricts admission of uncertain items while maintaining MAX_FREQ=5 retention, allowing the frequency gate to operate on an even more carefully curated set and sustaining the compounding effect.

## Hurt

(none)

## Takeaway

This variant extends the structure-frequency compounding effect to 1/15 selectivity with marginal but measurable gains. The tighter admission gate continues to filter out noise items before they consume main queue resources, and the pairing with MAX_FREQ=5 still sustains value for the items that clear admission. However, the diminishing rate of improvement (much smaller gain than v003 over its parent) signals that the peak of this interaction is approaching; further tightening of the admission ratio may risk evicting signal items too aggressively.
