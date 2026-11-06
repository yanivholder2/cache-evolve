# Variant v003 Insight

## Helped

- hit-rate: The hill climber starts by growing the window and only reverses when a sample's hit rate falls. At the small cache size it gained clearly on both web_0 and hm_0. This suggests v001's fixed 1% window was too small there: there was not enough room for newcomers to be re-referenced before they had to face the sketch contest against a main region full of stale, never-aged counts. At the large size, hm_0 also gained modestly.

## Hurt

- hit-rate: At the large size, web_0 effectively did not move (a negligible dip). The likely reason is that `sample_period = capacity`, so a 131072-entry cache gets about 8× fewer climb steps per trace than the small one. Each step is also judged on the hit-rate change between two noisy samples, so the window never moves far from where adapting would pay off. This is the opposite of what H03 predicted: it expected the gain to come mostly at the large size.

## Takeaway

On these MSR traces, window size matters a lot at small capacity, and v001's 1% was undersized there. An adaptive window recovers most of that loss without any per-trace tuning. At large capacity, tying the sampling period to `capacity` makes the climber too slow to help within one trace. The large-size gap v001 left open probably needs a shorter or capacity-independent sample period (or a different mechanism such as SLRU or aging), not just a resizable window. The lazy rebalance (evicting main LRU without a contest while the window grows) did not visibly hurt hit-rate, so it looks like a safe way to apply cap changes.
