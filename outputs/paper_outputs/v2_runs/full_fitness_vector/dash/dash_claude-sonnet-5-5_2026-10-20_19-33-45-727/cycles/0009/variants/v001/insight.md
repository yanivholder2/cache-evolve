# Variant v001 Insight

## Helped

- hit-rate: Halving the doorkeeper's aging threshold keeps the 512-bit filter sparser, so fewer one-hit keys pass as false second-hits and fewer cold entries displace useful ones. The gain is small.

## Hurt

- per-op-time: Both get and put got a little slower. Earlier resets mean the 8-word clear runs about twice as often, and each reset forgets true first touches. This may be partly measurement noise, since the hot path is otherwise unchanged.

## Takeaway

The aging threshold is a weak lever. Moving it from about 50% to about 25% fill bought a marginal hit-rate gain and cost a marginal latency increase. Metadata-size stayed at 40 because the layout is untouched. Further doorkeeper tuning is likely to yield only small trade-offs, and bigger gains probably need changes to eviction or promotion instead.
