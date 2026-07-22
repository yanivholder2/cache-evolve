# Variant v003 Insight

## Helped

- metadata-size: The bounded protected set is computed on the fly from `max_size` and stored entirely in the existing bucket-side `reuse: Vec<bool>` (no new per-`Item` field), so the cap machinery held `Item` at the 16-byte floor exactly as the hypothesis required.

## Hurt

- hit-rate: The core bet backfired — capping protection at `max(1, max_size/2)` and demoting the oldest protected slot on every over-cap hit forces genuinely-reused items back to probationary, where `eviction_candidate_index` then evicts them; the "keep a live probationary zone" intent instead threw away hot residents, dropping below the parent's 0.7267 rather than lifting above it.
- per-op-time: put p50 and p99 both rose because `get_from_position` now adds a full-vector `filter(...).count()` of set bits plus a second index-0 scan to find the demotion target on top of the pre-existing `realign_reuse` rebuild, layering more per-access linear work onto the already-quadratic hit path.

## Takeaway

Bounding the protected segment to half the bucket is too aggressive for these short associative buckets: demoting-then-evicting the least-recently-used protected slot evicts real reuse winners instead of scan-fodder, so the SLRU-style cap that helps long segments actively lowers hit-rate here. The "protection saturates to LRU" concern is real, but `max_size/2` overcorrects — any future cap must sit much closer to `max_size` to demote only when protection is near-total.
