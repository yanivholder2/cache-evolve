# Variant v001 Insight

## Helped

- per-op-time (get p99): The get tail dropped a lot even though H01 only restructured the evicting `put` path. That points to run-to-run noise or cache and allocator effects, not a real gain from the change.

## Hurt

- per-op-time (put): Put p50 and p99 both got slightly worse. Fusing the map remove and insert did not cut the cost of the evicting `put`. The extra probe or restructuring probably cost about as much as the map operation it removed.

## Takeaway

Reducing the number of hash-map calls in the evicting `put` did not make `put` faster. The map operations were not the bottleneck there, or the single-probe version costs about the same as remove plus insert. Hit-rate, metadata-size (24-byte nodes) and the per-op-time summary are all unchanged, so the policy and layout were preserved. Further per-op-time gains probably need to come from somewhere other than this map-call count.
