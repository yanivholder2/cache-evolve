# Variant v002 Insight

## Hurt

- hit-rate: Selective preservation of weak-signal items (freq < 4) prevents the cache from properly distinguishing between briefly-accessed and truly-hot items, allowing low-frequency items to persist longer than optimal and reducing eviction quality.

## Takeaway

Selective frequency decay based on thresholds—preserving weak signals (freq 0-3) while decaying strong signals (freq ≥ 4)—backfired. The strategy assumed that protecting newly-accessed items from decay would help them elevate faster, but instead it allows weak-frequency items to remain artificially high, degrading the cache's ability to discriminate between ephemeral and persistent access patterns. Uniform decay, even at the gentler 0.8× rate, maintains better frequency separation and hit-rate performance.
