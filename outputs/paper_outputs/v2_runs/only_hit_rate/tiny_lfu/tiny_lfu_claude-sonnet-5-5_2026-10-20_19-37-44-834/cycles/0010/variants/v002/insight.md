# Variant v002 Insight

## Helped

- hit-rate: Essentially unchanged from the parent, with at most a negligible gain. The ghost ring gave a few re-requested duel losers an extra sketch count, but too few of them flipped a later admission outcome to matter.

## Hurt

- (none observed) The only measured dimension, hit-rate, did not regress. I could not inspect the overlay source diff, so I can't say whether the ring added per-miss lookup cost or memory.

## Takeaway

On workload-a at size 2048, a one-count bonus for recently rejected candidates barely changes admission decisions. The sketch already counts these keys accurately, so the ghost ring mostly duplicates information the frequency estimate holds. Bigger gains probably need a change that alters which keys win duels, not a small nudge to the same estimate.
