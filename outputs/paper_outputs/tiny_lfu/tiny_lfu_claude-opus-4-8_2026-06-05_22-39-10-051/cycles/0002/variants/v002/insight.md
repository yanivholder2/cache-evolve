# Variant v002 Insight

## Helped

- (none): The doorkeeper-gated estimator did not improve any measured
  dimension; hit-rate fell below the v003 parent on workload-a.

## Hurt

- hit-rate: Gating `sketch.increment` behind the Bloom doorkeeper starves the
  main count-min counters — a recurring key earns no count-min weight until its
  second sighting *within the same doorkeeper window*, and the periodic clear
  every `8 * capacity` observations repeatedly wipes that window, so genuinely
  hot keys accumulate less long-term signal than they did when every access
  incremented the sketch. The flattened estimates make v003's forward-sample
  victim ranking and the admission gate less discriminating, slightly lowering
  hit-rate instead of raising it.

## Takeaway

The canonical TinyLFU doorkeeper is a net loss on this workload at this
capacity: filtering one-hit wonders out of the sketch also delays and resets
the frequency signal for recurring keys, and on workload-a the cost of the
weaker recurring-key estimate outweighs the benefit of keeping cold collisions
out of the cells. The `+1` doorkeeper floor in `frequency` cannot recover this,
because it gives the same one-point boost to true one-hit wonders and warming
hot keys alike. A larger doorkeeper window (less frequent clearing) or
restricting the doorkeeper to admission-only would be needed to test whether
the one-hit-wonder filter can pay off without dulling victim ranking.
