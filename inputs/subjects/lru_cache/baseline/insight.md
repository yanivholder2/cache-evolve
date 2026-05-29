# Baseline Insight

O(1) LRU cache backed by a `HashMap` plus an arena-based doubly-linked list.
Eviction slots are reused in place, so there is no free-list management; the only
configurable knob is `capacity`.

This is the seed baseline: the starting point every cycle-1 cohort is proposing
improvements over. Measured fitness lives in `eval/fitness-vector.json`.
