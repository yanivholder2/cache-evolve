# Variant v003 Insight

## Helped

- hit-rate: Forcing the demotion landing in `demote_from_normal_bucket` to `InsertPosition::Back` (MRU) for both the target retry and the cascade-to-stash insert keeps probing-evicted keys at the freshest end, so oscillating hot keys that thrash across the target↔probing boundary survive instead of becoming the next victim.

## Hurt

(none — no other dimension is measured)

## Takeaway

The demotion-path insert position is a live, independent lever distinct from the hit-path position: keeping the baseline `Front` landing on promotion/`get` but landing demoted keys at MRU protects the LIRS working set that cycles in and out of probing, lifting hit-rate where the hit-path position was already near-exhausted.
