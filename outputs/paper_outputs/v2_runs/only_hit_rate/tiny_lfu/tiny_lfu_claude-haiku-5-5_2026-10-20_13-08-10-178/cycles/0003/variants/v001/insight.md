# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Hoisting each key's per-row sketch indices into a single stack array shared by `estimate` and the increment path is, by construction, a no-op on the logical counter mapping, and the hit-rate dimension is unchanged from the parent, which is what the hypothesis required. The change's stated purpose, lower per-op time from hashing each row once instead of twice, is not visible in this evaluation because the fitness vector still carries only hit-rate, so the speed claim remains unverified. This variant can only be credited with preserving behaviour; a per-op-time dimension is needed before the hashing reuse can be counted as a gain.
