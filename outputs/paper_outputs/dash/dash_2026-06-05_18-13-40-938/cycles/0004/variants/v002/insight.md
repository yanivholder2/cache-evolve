# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Repositioning accessed items within stash on each access doesn't advance them to higher tiers where they would actually improve hit rate; the extra work of remove-and-reinsert interferes with eviction ordering without achieving promotion.

## Takeaway

MRU repositioning within the stash tier does not translate to hit-rate improvement. Items accessed in stash need to be *promoted* to target or probing to meaningfully impact hit rate; reshuffling their order within stash is orthogonal to the tier advancement that actually drives cache performance. The hypothesis confuses intra-tier ordering with cross-tier progression.
