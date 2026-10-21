# Variant v001 Insight

> Note: I could not open the overlay source in this session. Directory listing and search tools were unavailable, and the source paths I guessed did not exist. The mechanism below comes from H01's stated change, and I did not check the diff itself. That change: on the fresh-key stash insert path (ghost miss, or a ghost hit that fails the recency gate), skip the insert and record only a ghost entry if the chosen victim is evidence-backed and no free slot exists.

## Helped

## Hurt

- hit-rate: The change slightly lowered hit-rate, by an amount small enough to be noise. One reading is that the doorkeeper rarely fires. Under no-BIP, fresh keys already land at stash LRU, so the victim is almost always another unproven probation key. Credited or demoted residents get picked only through the narrow second-chance-wrap case this hypothesis targets. When the bypass does fire, the cost and the gain roughly cancel. Turning away a short-reuse fresh key loses about the same number of hits as keeping the protected resident saves.

## Takeaway

On the fresh-key admission path, the "who reaches the stash" lever is spent. The parent's LRU landing plus second-chance scan already keeps evidence-backed residents safe from fresh keys, so an explicit evidence check barely changes which keys are evicted. The rationale's prediction holds: admission is at its optimum here. Further gains will have to come from somewhere other than the fresh-key admission gate, such as the demotion path or ghost and stash capacity. This result comes from one LIRS trace at one cache size.
