# Variant v001 Insight

## Helped

- hit-rate: The single capacity-conserving widening step in `BaselineGeometry::layout` (`nb /= 2; bs *= 2`) doubled per-bucket associativity, so each collision set could hold more hot keys before evicting, and the segmented SLRU retained a slightly larger working set than under the parent's identity split.

## Hurt

- hit-rate: No regression — capacity was conserved exactly (`nb * bs` unchanged) and the bucket logic was left untouched, so the gain came purely from coarser routing rather than any change to admission or promotion.

## Takeaway

Layering one bucket-count halving on top of v003's best segmented logic confirms H01: at this geometry the cache was still mildly associativity-starved, and modestly wider collision sets let the promote-on-first scheme protect more of the hot set without forfeiting routing. The gain is small, suggesting the regime is near its associativity sweet spot — a second halving would likely trade routing away for diminishing returns, and the wider per-bucket scan cost it adds is invisible to this hit-rate-only fitness vector.
