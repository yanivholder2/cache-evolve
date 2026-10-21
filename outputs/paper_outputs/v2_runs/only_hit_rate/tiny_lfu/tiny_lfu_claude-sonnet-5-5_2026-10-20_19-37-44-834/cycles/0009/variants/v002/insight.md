# Variant v002 Insight

## Helped

- hit-rate: Breaking sketch-estimate ties with a `touched_in_window` flag gave a tiny improvement over the parent. A candidate that was re-referenced in the window is more likely to be reused, so admitting it on ties and rejecting untouched ones filters some one-shot entries. The gain is very small and within noise for a single trace and cache size.

## Hurt

- No reported dimension regressed. I could not list or diff the overlay source in this session, so this comparison rests on the hypothesis text and the fitness vectors.

## Takeaway

Recency-gating the tie case in admission is nearly neutral on workload-a. Ties apparently make up only a small share of admission duels, or the sketch already captures most of the signal the window-hit flag adds. Further hit-rate gains probably need to come from something other than refining the tie-break.
