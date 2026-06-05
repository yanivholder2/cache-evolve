# Variant v002 Insight

## Helped

- hit-rate: The bounded key-only ghost list re-admits a returning evicted key straight into the protected segment (via `push_front_protected`) instead of probation, so warm-but-evicted keys that the 90/10 split had churned out now survive their next access — but the lift over the parent is barely perceptible.

## Hurt

## Takeaway

Stacking the weak-protection ghost (`push_front_protected` at the protected-LRU end, plus `enforce_protected_budget`) onto the 90/10 SLRU winner moves hit-rate only marginally: with protection already pushed to 90%, genuinely hot keys are mostly retained anyway, leaving a vanishingly thin warm-but-evicted band for ghost re-admission to recover. The mechanism is directionally correct on this trace but its headroom is nearly exhausted by the incumbent split — and it pays for that sliver with a ghost structure and an O(n) `ghost_remove` queue scan on every new-key insert.
