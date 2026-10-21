# Variant v001 Insight

## Helped

- hit-rate: The hypothesis shrank the probationary segment from about 16 to 8 of 64 slots per bucket. That left more slots for protected, re-referenced entries, and hit-rate rose slightly over the parent. The gain is small, so the effect of the sizing change is modest.

## Hurt

- No dimension regressed. Only hit-rate was measured, so costs such as per-op-time and metadata-size are still unobserved. I could not inspect the overlay source here, so this reading rests on the hypothesis description and the fitness vectors.

## Takeaway

On this LIRS trace the optimal probation size is at or below an eighth of the bucket, not a quarter. Most of the gain from the probation/protected split comes from having the split at all, and tuning its size gives only a small further improvement. The next step is to test an even smaller probation segment, such as 4 slots, to find where the trend turns, or to start measuring per-op-time.
