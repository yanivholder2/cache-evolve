# Variant v001 Insight

## Helped

- hit-rate: Doubling `SKETCH_WIDTH_FACTOR` from 4 to 8 cut the per-row hash collision probability in half (sketch columns grow from 8 192 to 16 384 at capacity=2048), reducing residual frequency inflation that conservative update alone could not eliminate and letting the k=4 victim scan and PROMO_THRESHOLD gate act on slightly sharper estimates.

## Hurt

## Takeaway

Widening the sketch from 4× to 8× capacity yields a real but small hit-rate gain on top of the conservative-update baseline, confirming that the two mechanisms are complementary rather than redundant: at 4× the sketch was still experiencing enough collision noise to matter, but the marginal return from 8× is modest, suggesting most of the easy collision-reduction benefit was already captured by conservative update. Further sketch widening is likely to produce diminishing returns.
