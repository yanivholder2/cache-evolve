# Variant v002 Insight

## Helped

- None measured: H02 kept the packed layout, so metadata-size stayed flat against the parent, as the hypothesis intended.

## Hurt

- hit-rate: Gating the saturating counter increment to about one in four get hits means hot keys accumulate lower counts, and the full-scan lowest-counter victim choice then evicts them sooner, which is the retention loss the sampling was expected to avoid.
- per-op-time (get): Skipping three of four increments on the hit path did not lower get latency; both p50 and p99 rose against the parent, so the per-hit counter write was not the dominant cost on get in this layout.
- per-op-time (put): The hypothesis only changed the get-hit increment, so the put regression is not explained by the described change; it is either noise or a change in the overlay that the hypothesis does not describe.

## Takeaway

Sampling the counter on get hits cost retention without buying get latency, so the premise that the hit-path write dominates get cost does not hold for this packed layout. This reading comes from the hypothesis text and the fitness vectors only: the overlay source could not be read in this session, so I did not confirm the diff matches the hypothesis.
