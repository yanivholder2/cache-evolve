# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Restricting the reuse-counter increment to hits on entries last touched before the immediately preceding access produced no measurable hit-rate movement on LIRS workload-a at size 2048 relative to its parent, so filtering back-to-back re-hits does not appear to be what separates persistent reuse from burst reuse on this trace. The overlay sources could not be inspected in this pass (directory listing and file search were unavailable), so I could not confirm the last-touch field, the increment-site gating, or that the reset on demotion and eviction and the 0.75 occupancy branch were left unchanged. This describes the intended change, not a verified implementation, and one trace and one run cannot rule out a small effect.
