# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing the strict tail second-chance rotation with windowed lowest-`freq` selection regressed the hit-rate, because survivors in the window are decremented in place instead of being rotated to the front — so the recency refresh that the rotation gave to `freq > 0` residents is lost, and the move-to-front ordering v003 built up in `get()` is no longer reinforced on eviction.
- hit-rate: Scanning the `W = len/16` oldest residents and removing the coldest-by-`freq` there (ties toward the tail) lets the policy evict a buried `freq == 0` key while leaving warmer tail entries pinned near the eviction edge, draining the second-chance protection that previously kept reused keys resident a cycle longer.

## Takeaway

On this LIRS trace the *rotation* in the parent's `evict_main` was doing more work than the victim-sharpening: pushing surviving `freq > 0` entries back to the front refreshed their recency and compounded v003's move-to-front discipline. Picking the coldest within a tail window is a more "accurate" local choice but discards that refresh, so a sharper victim scan actively cost hit-rate rather than gaining it.
