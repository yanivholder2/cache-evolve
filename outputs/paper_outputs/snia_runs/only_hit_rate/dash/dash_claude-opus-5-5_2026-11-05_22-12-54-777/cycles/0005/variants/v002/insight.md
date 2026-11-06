# Variant v002 Insight

## Helped

- hit-rate (hm_0, size 16384): A ghost hit now admits at `remembered + 2` instead of a flat 1, so one stash-bucket halving no longer drops a returning key back to the level of one-touch arrivals. The gain shows up where the stash churns hardest, the small cache, because that is where many keys fall out of the stash and come back while the 64-slot ring still holds them. At size 131072 the change is noise-level, which fits: fewer keys drop out, so the ghost rarely fires, and a stronger bonus has nothing extra to act on.

## Hurt

- hit-rate (web_0): A small, consistent dip at both sizes. The parent showed web_0 was neutral to the ghost, so the cause is most likely in this change. Two plausible causes. First, the 12-bit fingerprint raises false matches about 16x (≈1.6% vs ≈0.1% per `put`), so scan keys sometimes get counter ≥2 by accident. Second, real ghost hits now carry a counter that survives one halving, so returning keys that never get reused again hold stash slots longer.
- (Not measured: the hypothesis asked for the false-positive rate to be reported with hit-rate, and for metadata-size and per-op-time to be checked as unchanged. This fitness vector has only hit-rate, so the false-positive cost is inferred, not observed.)

## Takeaway

A stronger admission bonus helps only where ghost hits happen often (hm_0 at the small size). It does not fix the parent's main limit: the ring remembers too few drops. Stash victims are picked from the low-counter tail, so the remembered count is probably near 0 most of the time. If so, the "rank by history" part adds little, and this variant behaves mostly like a bonus of 2 instead of 1. Meanwhile, cutting the fingerprint to 12 bits to make room for the count costs a little on scan-heavy web_0. Reach (`GHOST_SLOTS`) looks like the factor that binds, not bonus strength. If the count is kept, it should not come out of fingerprint bits.
