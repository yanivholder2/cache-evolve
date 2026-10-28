# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): At this size the climber starts with a larger first step, about 0.8% of capacity instead of about 0.2%. So the first few rounds move the window far enough to reach web_0's larger recency set sooner. The step still decays by 2% per round and resets to `step_init`, so the bigger start does not push the window out of bounds.
- hit-rate (summary): The two small-size cells exactly match the parent, and the two large-size cells reproduce the cycle 0006 large-step result. So the effects of gating `step_init` by size added up as the hypothesis predicted. The net result is a small but strict gain over the parent.

## Hurt

- hit-rate (hm_0 @ 131072): A marginal loss. A first step of 1024 makes the climber overshoot the right window size on this scan-heavy trace before the decay brings it back. hm_0's reuse set already fits in the main area at this size, so a bigger window only lets a few more scan entries in. The hypothesis expected this cost from cycle 0006 and chose to accept it.

## Takeaway

The best initial step for the climber depends on cache size. It is not one global constant. Gating `step_init` at a 32_768 threshold separates the large-cache web_0 gain from the hm_0 @ 16384 regression that a flat 1024 step caused. The small-size cells were not moved at all, which confirms that the change is contained. The large-size gain is the same as cycle 0006 at the cell level, and hm_0 @ 131072 still loses a little. So this lever is probably used up as a pure step change. More gains at 131072 will likely need something other than a bigger first step, such as a period or decay that depends on cache size, or a separate step size for shrinking the window.
