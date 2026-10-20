# Variant v002 Insight

## Helped

## Hurt

## Takeaway

This variant was meant to be an equivalence-preserving refactor: a per-bucket occupancy counter replacing the slot scans in `evict_one()`, with victim choice unchanged. Hit-rate stayed exactly at the parent's value, which is what the equivalence requirement predicted, but the variant's fitness vector records no per-op-time or metadata-size, so the claimed speedup is untested. The overlay source could not be inspected in this session, so this reading rests on the hypothesis and fitness data alone.
