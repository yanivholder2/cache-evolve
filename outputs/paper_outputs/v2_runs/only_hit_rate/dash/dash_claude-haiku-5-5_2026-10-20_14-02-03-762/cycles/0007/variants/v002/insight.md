# Variant v002 Insight

## Helped

- hit-rate: Shifting the reuse counter right by one on normal-tier demotion, instead of clearing it, is meant to keep partial reuse credit for keys that return soon after demotion. On LIRS workload-a at size 2048 the measured change over the parent is within noise, so this change did not produce a detectable gain.

## Hurt

## Takeaway

Halving the reuse counter on demotion, rather than zeroing it, made no detectable difference on workload-a at size 2048, so partial reuse credit across demotion is not a visible lever on this trace. This insight is limited: the overlay source was not readable in this pass (directory listing and file reads failed), so I could not confirm which reset site was changed or that the counter shift, the threshold, and the 0.75 occupancy branch match the hypothesis. Treat this as a reading of the fitness result, not of the implementation.
