# Variant v002 Insight

## Helped

- metadata-size: Repartitioning is pure buckets-vs-slots factoring of the same budget — no per-item state was added, so the 16-byte `Item` floor and the 24-byte cache header stay put exactly as inherited.

## Hurt

- hit-rate: Quartering `bucket_size` and quadrupling `num_of_buckets` in `layout` shrank per-set associativity, so hot keys that hash into the same now-narrower set have fewer resident slots to share and evict each other — conflict misses rose and hit-rate fell below the parent, the opposite of what the "set diversity beats per-set depth" note predicted.
- per-op-time: The `put` p99 tail widened because the lower hit-rate routes more operations down the heavier eviction/insert branch, and concentrating the same churn into smaller sets gives each hot key less depth to absorb a scan before it triggers a relocation.

## Takeaway

On this LIRS trace the untested "narrower sets" direction is a genuine loss, not just neutral: conserving the budget as 4× more, ¼-width sets trades away the per-set depth that keeps hot keys resident, so the finer hash partition raises conflict misses and regresses both hit-rate and the put tail. The SLRU ceiling here is guarded by per-set associativity, not broken by set count — the geometry seam's losing direction is confirmed to be *both* wider and narrower than the harness default.
