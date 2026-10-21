# Variant v001 Insight

## Helped

- (none observed in the measured dimensions)

## Hurt

- hit-rate: Halving the sketch aging threshold makes the "halve all counters" pass fire twice as often, so frequency estimates lose resolution sooner. The admission gate then discards useful history for items with a long reuse distance, and hit-rate slipped slightly below the parent's.

## Takeaway

Shortening the aging window did not help; it gave a marginal regression. Together with the earlier finding that a longer window was worse, this suggests the window is already near a flat optimum on workload-a, so tuning it is a weak lever. The conclusion rests on the hypothesis text and the fitness vectors. I could not read the overlay source in this session.
