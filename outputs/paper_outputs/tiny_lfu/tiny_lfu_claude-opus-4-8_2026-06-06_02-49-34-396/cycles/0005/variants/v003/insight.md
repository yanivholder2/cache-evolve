# Variant v003 Insight

## Helped

- hit-rate: Deepening the victim walk from 5 to 10 candidates lets the strictly-`<` `sketch.estimate` comparison find a genuinely colder entry more often on this Zipfian-ish trace, so hot entries survive eviction longer.

## Hurt

- per-op-time: Doubling the sampled candidates doubles the worst-case forward walk along `next` in `put`, lifting put p99 latency (the eviction-walk tail the cross-cycle notes already flagged); put p50 and the scored `get` p50 summary are untouched since the walk is put-only.

## Takeaway

For this trace, eviction quality is only weakly sample-depth-bound: doubling the victim sample buys a marginal hit-rate gain while measurably widening the put p99 tail. The sketch estimate, not sample depth, is the binding constraint — deeper sampling sharpens an already-near-saturated signal at real tail-latency cost.
