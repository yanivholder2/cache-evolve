# Variant v002 Insight

## Helped

_(no dimensions improved)_

## Hurt

_(no dimensions regressed)_

## Takeaway

Adding frequency-gated stash bypass — routing warm stash items (`lfu_counter >= 2`) directly into probing when it has a vacancy — produced no measurable change in hit-rate compared to the parent. The gate likely fires rarely in practice: items that accumulate high `lfu_counter` values while in stash are uncommon because the conditions that drive items into stash (target-full cascades) tend to involve items that were not yet warm when they fell. When the gate does fire, collapsing two re-promotion steps into one is too infrequent to shift aggregate hit-rate on this trace, indicating the stash→target→probing path is not the bottleneck the hypothesis assumed.
