# Variant v003 Insight

## Helped

- hit-rate: Putting main-queue evictees into the ghost, shrinking the small queue to about 5% of capacity, and shortening the ghost to about 0.5x main size gave a hit-rate gain over baseline on workload-a. The gain comes from a smaller probationary area, which quickly filters one-hit wonders. A shorter ghost then limits stale re-admissions. A ghost hit goes straight to main with freq=0 and no per-key flags.

## Hurt

- metadata-size: Not measured for this variant, so no effect can be claimed. The hypothesis adds no per-key ghost state, so metadata should stay near baseline, but that is unverified.
- per-op-time: Not measured for this variant, so no effect can be claimed. Extra ghost inserts on main eviction add work to `put`, but their cost was never evaluated.

## Takeaway

Three small changes applied to the baseline reproduce a hit-rate gain without any added per-key state. That supports using this smaller diff as a cleaner parent. The fitness vector only covers hit-rate, so the cost side (metadata and per-op time) is unconfirmed. I could not read the overlay source in this session, so this interpretation rests on the hypothesis text rather than the code diff.
