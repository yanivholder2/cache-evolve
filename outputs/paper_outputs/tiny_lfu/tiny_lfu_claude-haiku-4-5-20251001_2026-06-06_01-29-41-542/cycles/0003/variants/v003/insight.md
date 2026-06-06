# Variant v003 Insight

## Hurt

- hit-rate: The dual-tier promotion strategy bypasses the sketch gate for hot-tier items, allowing recently-accessed items to be unconditionally admitted even when their sketch frequency is lower than the victim's, introducing lower-quality admission decisions.

## Takeaway

The dual-tier fast-path optimization underperformed because its admission logic diverged from the baseline's sketch-based gate. By unconditionally admitting items promoted to the hot tier, the variant sacrifices the frequency comparison that ensures only items with higher access patterns displace victims. This breaks the core eviction discipline without delivering sufficient reduction in sketch collision noise to offset the degraded gate quality.
