# Variant v003 Insight

## Helped

- hit-rate: Demoted items inserted at Back (MRU) position restore immediate re-promotion eligibility, allowing items that climbed the tier hierarchy to quickly return if accessed again, reducing the cost of demotion cycles.

## Hurt

## Takeaway

Moving demoted items from front-insertion (LRU) to back-insertion (MRU) in the demotion cascade creates a faster re-promotion path for items proven valuable by their tier advancement. Rather than forcing recently-demoted items to wait behind other residents before earning re-promotion, they immediately compete for next access, preserving their signal value across tier transitions. This asymmetry—penalizing fresh evictions but rewarding items with proven history—boosts hit rate by keeping hot items cycling through tiers more efficiently.
