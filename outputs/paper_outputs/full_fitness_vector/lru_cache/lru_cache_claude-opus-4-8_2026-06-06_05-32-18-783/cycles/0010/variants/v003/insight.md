# Variant v003 Insight

## Helped

- (none) — enlarging probation did not buy any dimension; the change was a pure cost on this trace.

## Hurt

- hit-rate: Shrinking `prot_cap` from `capacity * 4 / 5` to `capacity * 3 / 4` discarded proven-hot keys from the protected segment faster than the roomier probation rescued short-reuse repeaters, so the net retention worsened — directly refuting the hypothesis's "probation-limited" premise.
- per-op-time: A smaller protected cap means `prot_len` crosses `prot_cap` sooner and stays saturated, so `demote_protected_lru` fires on far more promotions, adding extra link surgery to both the get (promote-on-hit) and put hot paths.

## Takeaway

On this short-reuse trace, protected-side retention — not probation eviction — is the binding constraint: taking ~5% of capacity away from the protected segment lost hot keys instead of saving repeaters, confirming the hypothesis's refute condition. Worse, the smaller cap pushes the protected segment into permanent overflow, so demote-on-overflow runs on nearly every promotion and taxes get/put latency for no hit-rate gain.
