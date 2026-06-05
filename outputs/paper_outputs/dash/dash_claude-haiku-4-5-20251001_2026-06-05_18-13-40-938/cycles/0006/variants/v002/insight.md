# Variant v002 Insight

## Helped

- hit-rate: Skewing bucket allocation to 18 probing buckets (288 slots) versus 10 target (160 slots) created more capacity for hot-tier residents by reducing eviction pressure from saturation-driven cascades.

## Hurt

## Takeaway

The skewed distribution confirms probing-tier capacity is a real but secondary bottleneck. The modest +0.0014 hit-rate gain—despite shrinking target capacity from 224 to 160 slots with no measurable penalty—reveals that the target tier was not saturated. This points to a more subtle constraint on reaching the 0.684 ceiling: the issue likely resides in promotion discipline, stash filtering effectiveness, or bucket assignment strategy rather than raw capacity limits.
