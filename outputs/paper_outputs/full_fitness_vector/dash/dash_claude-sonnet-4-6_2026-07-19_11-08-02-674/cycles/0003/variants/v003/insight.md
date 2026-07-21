# Variant v003 Insight

## Helped

- **hit-rate**: The vacancy gate lets frequency-qualifying non-hottest items ascend to probing when space exists, providing a marginal lift by accelerating promotion for items with demonstrated reuse that would otherwise wait for the hottest-item criterion.

## Hurt

*(no dimensions moved negatively)*

## Takeaway

Adding vacancy-gated frequency promotion on top of the parent's plain MRU-end promotion preserved latency (no regression at the per-op-time summary level) while delivering a fractional hit-rate improvement, confirming that tying gate-fires to probing vacancy structurally blocks cascade evictions. However, the cascade-elimination guarantee did not produce a measurable latency *gain* either — the get p99 was unchanged — suggesting that on this workload the frequency-qualifying promotion events are rare enough that their cumulative cascade cost was already negligible, and the gate's main practical contribution was the marginal hit-rate benefit from accelerating select non-hottest items to probing sooner.
