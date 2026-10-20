# Variant v001 Insight

## Helped

- metadata-size: Single-step promotion is pure ordering (`items.swap(position, position + 1)`), so `Item` stayed at the 16-byte key+value floor with no flag or counter added — the change bought no state, but it also cost none.

## Hurt

- hit-rate: Overriding `get_from_position` to nudge a hit only one slot toward the back regressed hit-rate below the move-to-back incumbent; in the small associative buckets a genuinely reused key needs several hits to ratchet out of the front eviction zone, and front admission + front eviction claim it before it climbs, so single-step under-protects real reuse rather than just filtering scan noise.
- per-op-time: The measured get/put percentiles rose even though the inline single swap does strictly less work than the parent's trait-dispatched remove-and-push-to-back, so the increase is not attributable to the promotion geometry and reads as variance on the tiny per-bucket vectors.

## Takeaway

The hypothesis — that first-touch move-to-back over-rewards one-hit scan noise — did not hold for this workload: on ClassicLRU's short buckets, full promotion on the first hit is what lets a reused key survive the next conflicting front insert, and demoting it to a one-slot climb let hot keys be evicted before they earned protection. Gradual, frequency-like promotion needs more slots (or a longer bucket) than the associative cache gives it to beat immediate MRU promotion.
