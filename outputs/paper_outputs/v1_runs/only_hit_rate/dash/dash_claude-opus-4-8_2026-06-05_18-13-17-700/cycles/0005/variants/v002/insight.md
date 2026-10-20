# Variant v002 Insight

## Helped

- hit-rate: No improvement — the position-differentiation lever moved this dimension the wrong way, so nothing here.

## Hurt

- hit-rate: Overriding `put` to admit cold keys at the stash LRU end (`InsertPosition::Front`, index 0) makes every freshly admitted key the *immediate* eviction victim on the next collision in its stash bucket, so a cold key is dropped before it can catch the second reference that would promote it to target — gutting the stash's protective admission value that cycle 0004 v002 proved is load-bearing.

## Takeaway

The hypothesis assumed cold keys and warm demotions are competing populations to be ranked, but on this LIRS trace the stash's value is giving *new* keys enough residency to prove themselves: front-inserting cold admissions denies them that residency entirely, evicting one-hit-candidates before they can become two-hit working-set keys. Differentiating by stash insert position is the wrong lever — protecting demotions by sacrificing cold-key dwell time costs more hits than it saves.
