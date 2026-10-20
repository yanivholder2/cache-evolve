# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Raising the small-queue promotion bar from two hits to three (H02) left hit-rate on the lirs workload-a trace at size 2048 exactly where the parent was, so this change did not move the one dimension measured. The fitness vector only reports hit-rate, so metadata-size and per-op-time remain unassessed. I could not open the overlay source to confirm the `freq > 2` edit actually landed, so the result cannot be tied to the code with certainty.
