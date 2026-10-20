# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Capping the per-entry hit counter at 3 (u8 with saturation, per H02) left hit-rate on workload-a at size 2048 within noise of the parent, which supports the hypothesis's claim that the victim window rarely needs counts above 3 to separate candidates. The intended benefit, a smaller per-node footprint, is not reported by the current fitness vector, so this variant does not show a gain and should not be kept on the strength of hit-rate alone. The source diff could not be inspected during this write-up, so the claim about the exact field change rests on the hypothesis text, not on the overlay code.
