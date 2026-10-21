# Variant v002 Insight

## Helped

- (none): the only measured dimension, hit-rate, did not improve over the parent.

## Hurt

- hit-rate: Stretching the sketch aging window to 40x capacity makes frequency counters decay more slowly, so stale high-count keys stay in the sketch longer and block newer candidates at admission. Moving doorkeeper clearing to its own 10x cycle decoupled it from halving but didn't offset this. On workload-a at size 2048 the result is slightly worse than the parent.

## Takeaway

On this trace, the extra frequency history from a 4x longer aging window did not capture useful long-distance reuse. The cost was slower adaptation to shifting popularity. Decoupling the doorkeeper reset from halving was not enough to turn a longer window into a gain. The aging period looks like it should stay near 10x capacity, or be tuned in a smaller step. This result also can't separate the two changes, because the window length and the door-reset change landed together.
