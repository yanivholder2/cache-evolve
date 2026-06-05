# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Extending the reset interval to capacity × 24 allows stale frequency signals to persist 50% longer before decay, degrading the cache's ability to distinguish truly hot items from items that were popular in earlier workload phases.

## Takeaway

Increasing the frequency decay reset interval from capacity × 16 to capacity × 24 produces a measurable regression in hit-rate. The capacity × 16 threshold represents a finely-tuned balance between preserving frequency history and clearing stale signals; a 50% longer interval tips this balance toward holding outdated information, which interferes with correct eviction decisions when workload access patterns shift.
