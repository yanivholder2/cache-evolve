# Variant v003 Insight

## Helped

## Hurt

## Takeaway

The per-bucket occupancy counter left hit-rate on the measured trace unchanged from the parent, which is what the hypothesis required, so the change is hit-rate-neutral here. Its intended effect, lower per-op-time from skipping the slot scan in `evict_one()`, could not be assessed because this variant's fitness vector records only hit-rate and omits per-op-time and metadata-size. The overlay source was not inspected in this pass, so the code-level claim that the counter replaced the scan is taken from the hypothesis rather than verified in the diff.
