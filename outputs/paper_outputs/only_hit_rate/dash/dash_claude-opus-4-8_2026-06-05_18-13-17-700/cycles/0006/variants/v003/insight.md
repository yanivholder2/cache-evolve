# Variant v003 Insight

## Helped

(none — no dimension improved)

## Hurt

- hit-rate: Promoting **both** residents of a `len == 2` target bucket (returning `true` for any `item_idx`) instead of only the hottest admitted the colder second resident into probing, which displaced more useful keys than it retained and nudged hit-rate down rather than up.

## Takeaway

This variant falsifies the hypothesis and validates v001's previously-asserted-but-unmeasured premise: the withheld second resident of a `len == 2` target bucket is *not* a warm key being needlessly held back — loosening that boundary over-admits and very slightly hurts. Because `len == 2` is the densest size with rank ambiguity, this is a clean read that the hottest-only threshold on small target buckets is doing real work; future hit-rate gains must come from denser buckets, not from relaxing the rank gate.
