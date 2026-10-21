# Variant v002 Insight

## Helped

- hit-rate: Hits on entries already above the len/4 slot now jump two slots toward the head. Repeatedly hit entries reach the protected front faster, so they are less likely to be evicted. The gain over plain transpose-by-one is small.

## Hurt

- per-op-time: The extra position compare and the two-slot swap on the hit path add work to `get`. Latency got worse for both `get` and `put`, and the `put` increase is larger than the code change alone explains. This may be measurement noise or a side effect on the layout that `put` works with. I did not read the overlay source, so I can't confirm which.

## Takeaway

Using position as an implicit "hit before" signal costs no metadata, and metadata-size stays at 40. It buys only a marginal hit-rate gain, paid for with a clearly slower hot path. Gating a bigger promotion jump by position is a weak trade here. It would need a larger hit-rate gain, or a cheaper way to branch, to be worth the latency.
