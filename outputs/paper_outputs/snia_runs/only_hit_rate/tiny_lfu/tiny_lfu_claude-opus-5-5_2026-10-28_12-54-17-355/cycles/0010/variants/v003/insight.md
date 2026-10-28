# Variant v003 Insight

## Helped

_None. No cell improved in a meaningful way. hm_0 small was essentially flat, which means the protected-split climber was not doing measurable work at 16384 on hm_0. Fixing `protected_pct` at 80 costs nothing there._

## Hurt

- hit-rate: hm_0 large took the biggest drop. Fixing `protected_pct` at 80 removed the protected climber's ability to move the probation/protected boundary. At 131072 on hm_0, that boundary was doing real work. Under H03's own test, a drop on hm_0 large means the c4 gain from the adaptive split was real, not noise.
- hit-rate: web_0 large also fell, so the expected gain from a "twice-as-fast" window climber did not appear in either large cell. One likely reason: `STEP_DECAY` is applied once per climb, so running the window climber on every sample also makes its step decay twice as fast per request. The extra climbs reach the 1-entry floor sooner instead of covering more ground. Comparing each sample with the one just before it also exposes reversals to sample-to-sample noise. Alternation used to smooth that noise over two periods.
- hit-rate: web_0 small slipped slightly. Fixing the split takes away a small extra adjustment that the window climber alone does not recover.

## Takeaway

Removing the protected-split climber and giving its turns to the window climber made the summary slightly worse. The losses were concentrated in the large cells, especially hm_0 at 131072. So the adaptive protected split was doing real work, even though its step had shrunk to 1-point moves. The window climber was not limited by having only half the samples. Giving it every sample, with per-climb decay unchanged, only decays its step faster per request. Future climber work should keep the protected split adaptive. If window speed is the target, the step decay or sample period should be retuned directly instead of reassigning turns.
