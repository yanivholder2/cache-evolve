# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Relocating the seen-in-stash marker into a spare tag bit left hit-rate exactly where v001 had it on LIRS workload-a at size 2048, which is the outcome a pure layout change should produce, so the change did not disturb the MRU landing or second-hit gate. The intended metadata-size reduction is not measured by any dimension in the fitness vector, so this variant's benefit (if any) is unconfirmed, and the eval artifacts alone cannot show that the bit was actually packed rather than the layout left untouched.
