# Variant v003 Insight

## Helped

- hit-rate: No movement — and that is the win the change was after. The added fast path in `get_from_position` (skip `remove(position)`+`push` when `position == len - 1` and `lfu_counter >= 1`) only elides a memmove that restored a byte-identical vector, so admission, promotion, and eviction order are provably untouched. The targeted gain is per-op-time, which the fitness vector does not measure.

## Hurt

- hit-rate: No regression; the elided `remove`+`push` case was a guaranteed no-op, so the segmented ordering is preserved exactly at the incumbent value.

## Takeaway

The MRU-back-and-protected branch is the hottest path in this SLRU bucket (high-reuse keys are re-hit there repeatedly), and v003 shows that branch can be reduced to a bounds check plus a counter read with zero behavioral cost — but because the cohort's only fitness axis is hit-rate, a purely cost-side win like this lands as a flat, undifferentiated tie. To reward this kind of change, selection needs a per-op-time dimension; without one, H03's banked work is correct but invisible.
