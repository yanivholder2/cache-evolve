# Baseline Insight

Vanilla Dash Segment with classic-LRU eviction, no custom bucket-movement
beyond the default promote-on-hit path inherited from `PromotionColdSegment`.

This is the seed baseline: the starting point every cycle-1 cohort is
proposing improvements over. Measured fitness lives in `eval/fitness-vector.json`
(produced by `evolve/scripts/measure-baseline.py`).
