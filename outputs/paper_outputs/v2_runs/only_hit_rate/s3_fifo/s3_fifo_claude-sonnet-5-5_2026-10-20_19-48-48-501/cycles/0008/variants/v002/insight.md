# Variant v002 Insight

## Helped

- hit-rate: Raising the frequency saturation cap to 7 lets hot items keep more reuse history, so main-queue second-chance passes protect them longer. The gain over the parent is very small.

## Hurt

- (none) hit-rate was the only dimension measured, and it did not regress.

## Takeaway

Raising the frequency cap, with the promotion gate, decrement-on-reinsertion, small size and ghost length unchanged, is a safe move in the right direction. On this trace it gives only marginal benefit, so the cap is close to saturated as a tuning lever. The hit-rate gain is probably within noise, and larger gains would need changes elsewhere, such as the promotion policy or queue sizing. I could not read the overlay source in this session, so this interpretation rests on the hypothesis text and the fitness vectors.
