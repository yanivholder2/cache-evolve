# Variant v001 Insight

## Helped

- hit-rate: Adding a density-independent per-key touch counter (`target_touches`, promote on `count >= 2`) as a second trigger next to `is_in_top_quarter` lets a genuinely warm key promote out of a len-1/len-2 target bucket where the rank gate degenerates to hottest-only, producing a barely-positive nudge over the parent's rank-only gate.

## Hurt

- hit-rate: The nudge is essentially noise-level — the touch counter fires on exactly the small buckets the hypothesis targeted, yet promoting those keys does not net new hits, because on this trace a warm key already reaches probing via the two-speed climb within an access or two, and the keys the counter additionally promotes displace residents of comparable warmth.

## Takeaway

Replacing the *rank* warmth proxy with a density-independent touch counter directly tests whether the small-bucket occupancy ceiling is structural or a rank artifact — and the flat result answers: the ceiling is structural. The warm/cold boundary on this heavily-colliding trace simply does not depend on which promotion signal you use; once the two-speed first hop is in place, a second trigger that fires on `>=2` target touches promotes keys the cache was already serving, so it neither helps nor hurts. This closes off touch-counting as a lever and points the search away from promotion-gate variations entirely.
