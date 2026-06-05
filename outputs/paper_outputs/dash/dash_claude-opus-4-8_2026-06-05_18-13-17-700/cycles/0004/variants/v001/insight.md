# Variant v001 Insight

## Helped

- hit-rate: Replacing the single-hottest `is_hottest` gate with `is_in_top_quarter` (`item_idx >= len - len/4`) lets every key in the freshest quarter of a target bucket promote into probing, so the protected probing tier fills with the whole warm set instead of one winner per target — a marginal hit-rate gain on the skewed trace. It compounds with v002's inherited `InsertPosition::Back`, which carries a key into that freshest quarter in a single access.

## Hurt

- hit-rate: The gain is only a sliver, because the `len < 4` guard falls back to the hottest-only test, and on this trace heavy hashing keeps most target buckets small enough that the widened branch rarely engages — so the lever moves far less than the hypothesis hoped.

## Takeaway

Widening the promotion gate from strict-hottest to top-quarter is directionally correct — admitting more warm keys into the probing tier nudges hit-rate up — but the `len < 4` small-bucket fallback caps its reach: on a trace where target buckets seldom hold four-plus residents, the quarter rule degenerates back to the baseline for most accesses. The gate lever is real but bottlenecked by bucket occupancy; unlocking it would require either denser target buckets or a fractional rule that engages below len 4.
