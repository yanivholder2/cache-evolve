# Variant v001 Insight

## Helped

## Hurt

- per-op-time: Get and put p50 both rose relative to the parent (cycle 0005 v001), the opposite of what H01 predicted from replacing the per-row seeded mixing with a module-level premixed constant table. The change apparently did not remove enough per-row work to offset whatever the new row-index path costs, or the single run is dominated by noise: p99 for both ops is far above the parent's, which suggests an unstable measurement rather than a stable regression.

## Takeaway

Premixing the per-row seeds into a `const` table left hit-rate and metadata-size unchanged, which is consistent with the new row indices matching the public per-row indices, but it did not make the per-row index path faster. The per-op-time result points against the rationale that the mixing cost sits in the hot loop. The overlay source could not be inspected in this session, and the fitness vector records only one run where the hypothesis required at least three, so the slowdown is not yet separated from noise and should be re-measured before the direction is treated as settled.
