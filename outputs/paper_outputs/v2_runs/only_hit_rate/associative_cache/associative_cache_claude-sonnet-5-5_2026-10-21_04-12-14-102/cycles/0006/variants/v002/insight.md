# Variant v002 Insight

## Helped

- (none measurable): The fitness vector only reports hit-rate. The rotate-based promotion/demotion this variant introduced targets per-op-time, and that dimension was not recorded, so any benefit is unverified.

## Hurt

- (none): hit-rate is identical to the parent. This matches the hypothesis's requirement that slot order and the chosen victim stay the same as v001.

## Takeaway

Replacing swap loops with single `rotate_*` calls kept the eviction policy exactly the same, so the rewrite is safe for hit-rate. The evaluation never measured per-op-time or metadata-size, which were the stated goals, so the hypothesis is neither confirmed nor refuted. Judging a pure-performance refactor needs a timing dimension in the fitness vector, because hit-rate alone can't distinguish it from its parent.

I couldn't list or diff the overlay sources in this session, because only Read, Write, Edit and Grep were available. This insight rests on the hypothesis text and the two fitness vectors, not on the code diff.
