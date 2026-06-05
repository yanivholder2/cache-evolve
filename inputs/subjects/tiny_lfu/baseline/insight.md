# Baseline Insight

TinyLFU cache: an arena-backed LRU resident set fronted by a count-min frequency
sketch. On a full cache, a new key is admitted only when its estimated frequency
exceeds the LRU victim's; otherwise the victim is kept and the newcomer dropped.
The only configurable knob is `capacity` (the sketch width is derived from it).

This is the seed baseline: the starting point every cycle-1 cohort is proposing
improvements over. Measured fitness lives in `eval/fitness-vector.json`.
