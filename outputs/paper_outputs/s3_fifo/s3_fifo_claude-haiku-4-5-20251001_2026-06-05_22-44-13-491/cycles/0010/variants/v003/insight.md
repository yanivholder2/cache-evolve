# Variant v003 Insight

## Helped

- hit-rate: Combining aggressive small-queue selectivity (1/15 sizing) with extended frequency protection (MAX_FREQ=6) still improves overall hit-rate over the baseline by filtering cold items faster while extending lifespans for hot items that reach the main queue.

## Hurt

- hit-rate: The pairing underperforms relative to either innovation in isolation; aggressive filtering of the smaller small queue (1/15) reduces admission of items to the main queue, limiting the practical benefit of the higher frequency ceiling (MAX_FREQ=6) that was designed to protect more items.

## Takeaway

Pairing structural selectivity with frequency depth reveals diminishing returns rather than synergy. The two optimizations pursue orthogonal strategies—one filters strictly upstream, the other protects generously downstream—and their combination exposes a tension: the more selective admission narrows the pool that can benefit from extended protection. Specializing in a single dimension outperforms trying to win on both fronts simultaneously.
