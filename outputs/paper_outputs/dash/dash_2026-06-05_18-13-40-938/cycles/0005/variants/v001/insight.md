# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Reverting promotion paths to Front insertion (LRU) while preserving demotion-path Back insertion (MRU) yields no change in performance—the hit rate holds steady at 0.684. This indicates that the saturation observed in the parent variant is not caused by back insertion on promotion paths destabilizing the probing tier. Rather, demotion-path back insertion appears to be the stable component that maintains the baseline, while promotion-path insertion position (Front vs. Back) has minimal direct impact on hit rate. The asymmetry hypothesis—that freshly promoted items at MRU in probing compete poorly—finds no support; performance neither improves when Front insertion is restored, nor regresses. This suggests the ceiling at 0.684 is governed by deeper architectural factors (tier occupancy balance, eviction policy interaction, bucket saturation dynamics) rather than insertion position asymmetry.
