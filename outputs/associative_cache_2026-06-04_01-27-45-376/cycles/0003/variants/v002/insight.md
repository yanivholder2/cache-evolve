# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Sparing back-half untouched slots forces the fallback to evict the touched survivor at index 0, so when a bucket holds only back-half newcomers, the policy sacrifices a proven re-referenced item instead of the never-touched one the parent would have killed.
- hit-rate: A freshly-inserted item at the back of the bucket is not predominantly "hot" on this trace — granting it grace mostly protects one-shot churn that the parent's full-bucket untouched scan was correctly culling.

## Takeaway

For ClassicLRU on workload-a, "`lfu_counter == 0` anywhere in the bucket" is already the right victim signal — positionally gating it splits the untouched set into a protected back half and an unprotected front half, but the back-half slots are the ones most likely to be one-shot arrivals (LRU just pushed hits to the rear), so the gate protects the wrong half. Bucket position alone, without a per-slot age or arrival-tick, cannot distinguish a hot newcomer from a cold one — it inverts the parent's signal rather than refining it.
