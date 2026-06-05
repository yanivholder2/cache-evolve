# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The two-pass victim selection refinement successfully targets a specific eviction bias: when multiple items share the minimum frequency, the parent's one-pass scan was biased toward selecting the newest matching item it encountered, while v001's two-pass approach selects the oldest (LRU) item instead. However, this refinement produced no measurable change in hit-rate. The decay mechanism appears to prevent frequencies from tying frequently enough for this eviction tiebreaker to matter on the workload, or the access pattern simply doesn't expose scenarios where oldest vs. newest selection within the minimum-frequency tier affects overall cache performance.
