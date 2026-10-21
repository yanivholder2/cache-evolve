# Variant v003 Insight

## Helped

(No measured dimension improved. The fitness vector reports only hit-rate, so metadata-size and per-op-time could not be compared with v001. The climber adds six scalar fields and a per-`get` counter and branch. `resize_window` can also relink up to `delta` nodes once per sample period. Neither cost was measured.)

## Hurt

- hit-rate: The hill-climber moved the window boundary away from v001's fixed 1%, and on workload-a that cost hits instead of recovering them. Several parts of the change point toward a larger, noisier window:
  - **Biased first climbs.** `prev_hit_rate` starts at 0.0 and `direction` at +1, and the hit rate rises naturally while the cache fills. So the first climbs reward "grow" by 128-slot steps whether or not a larger window actually helps.
  - **Noisy direction.** Each decision compares one sample against one sample with no smoothing or threshold. Phase noise in a LIRS-style trace therefore flips `direction` at random, and the 0.98 step decay is too slow to damp that walk in time.
  - **Larger LRU region.** A larger window turns more of the capacity into a plain LRU region. That is the policy LIRS-style loop/scan traces punish most, and it shrinks the frequency-filtered probation and protected segments.
  - **No admission check on shrink.** When the window shrinks, `resize_window` moves window LRU entries straight into probation without running `contest`. Cold keys skip the admission filter and can push out real probation entries.

## Takeaway

On this trace, an adaptive window built from a naive hill-climber does worse than a fixed 1% window. The climber is unseeded (`prev_hit_rate = 0`, starting direction grow), measures its signal from raw single-sample hit-rate deltas, and starts with large steps, so it drifts toward window sizes the trace doesn't reward. That contradicts v001's guess that the tiny window was the bottleneck. If window adaptation is tried again, it needs these fixes:

- start the climber only after warm-up;
- add a hysteresis/threshold on hit-rate deltas;
- use much smaller initial steps;
- run the contest for keys pushed out of the window on a shrink.

Otherwise, adaptation should be dropped in favour of the sketch-aging lever v001 identified, which this change left untouched.
