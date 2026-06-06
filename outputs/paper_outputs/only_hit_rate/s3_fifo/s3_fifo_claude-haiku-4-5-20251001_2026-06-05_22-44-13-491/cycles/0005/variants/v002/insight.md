# Variant v002 Insight

## Helped

- **hit-rate**: Further tightening the small-queue admission threshold from 1/8 to 1/10 of capacity continues to improve selectivity, maintaining the monotonic improvement trajectory and indicating that the working-set diversity still supports a more aggressive filtering gate without triggering utilization saturation.

## Hurt

(none)

## Takeaway

Reducing the small-queue admission ratio from 1/8 to 1/10 extends the pattern of improvement observed across cycles 0002–0004, confirming that S3-FIFO's performance on this workload scales with tighter admission selectivity. The positive result demonstrates that the saturation boundary lies beyond 1/10 of capacity—the workload's reuse patterns remain sparse enough that an even narrower gate still accurately captures items worthy of main-queue promotion (freq > 1), without artificially constraining useful working-set diversity. This variant advances the boundary-discovery hypothesis and suggests optimal sizing may be tighter still, warranting continued refinement in future cycles.
