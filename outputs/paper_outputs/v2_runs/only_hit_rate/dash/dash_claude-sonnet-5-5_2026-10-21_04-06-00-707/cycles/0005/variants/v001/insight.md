# Variant v001 Insight

## Helped

- hit-rate: Per the hypothesis, the second-sighting filter got 4x more bits per segment and no periodic reset. That means fewer false "seen before" collisions and fewer premature forgets. The result is a very small gain over the parent on workload-a, within what one trace could plausibly produce as noise.

## Hurt

- None measured. hit-rate is the only reported dimension. The 4x larger per-segment filter state costs more memory, and that cost is not captured. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

The parent's admit-on-second-sighting rule was already close to saturated with respect to filter capacity and aging. Making the filter 4x bigger and never clearing it barely changed outcomes. The remaining hit-rate headroom is therefore not in filter precision, and further gains probably need a different admission signal rather than a better-tuned bit array. Since memory grew for a negligible gain, the trade is probably not worth keeping.
