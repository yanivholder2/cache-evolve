# Variant v001 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Adaptive counter recycling—dividing all sketch counters by 2 when saturation occurs—delivered no measurable improvement despite the theoretical rationale that it should preserve frequency discrimination at high occupancy. The hit-rate remained flat at 0.7415. This suggests that on this workload, counter saturation is either a rare event that doesn't materially affect admission decisions, or the loss of absolute frequency precision is offset by collateral damage from the global right-shift reducing confidence in all frequency estimates, not just saturated ones. The recycling strategy trades raw counter precision for continued accumulation, but the 16x-width sketch may already provide enough discrimination width that both approaches yield equivalent decisions.
