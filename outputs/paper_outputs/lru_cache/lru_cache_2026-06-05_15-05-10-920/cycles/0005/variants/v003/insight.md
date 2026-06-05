# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The exponential decay approximation (74/100 = 0.74) is slightly more aggressive than the parent's multiplicative 0.75, causing frequency scores to decay faster and reducing the preservation of high-frequency items across phase transitions.

## Takeaway

Replacing multiplicative decay with an exponential approximation results in a marginal regression. While exponential decay is theoretically sound for creating a smoother forgetting curve that should benefit mid-range frequency items, the integer approximation (0.74) becomes slightly more aggressive than the prior coefficient (0.75), which undermines the very benefit the parent variant gained: maintaining frequency discrimination through phase shifts. The added decay cost outweighs any theoretical benefit from the exponential shape.
