# Variant v003 Insight

## Helped

- hit-rate: Shrinking the small queue to ~5% of capacity and shortening the ghost to ~0.5x main sends fewer one-hit objects through a large probation area, leaving more room for the reused working set. This improved on the baseline even though main-queue evictees are not ghosted.

## Hurt

- metadata-size / per-op-time: The fitness vector reports no values for these dimensions, so this variant's effect on them is unknown. The change only alters capacity constants, so a large shift is unlikely, but that is not measured.

## Takeaway

Sizing alone (small queue ~5%, shorter ghost) produces a hit-rate gain over baseline on this trace, with no main-evictee ghosting. Ghosting main-queue evictees is therefore not required for the gain, though this variant cannot show how much extra it contributes. I could not list or read the overlay source in this session, so this interpretation rests on the hypothesis description and the fitness vectors.
