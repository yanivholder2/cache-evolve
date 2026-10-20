# Variant v001 Insight

## Helped

- hit-rate: Counting each access once and lowering the admit threshold from 2 to 1 (ties admit) essentially matched the double-count parent, with a marginal gain. The lower threshold offset the lost second increment, which supports the hypothesis that the two knobs are coupled. The gain is within noise for a single trace and size.

## Hurt

- No reported dimension regressed. I could not list or diff the overlay source in this session, so this comparison rests on the hypothesis text and the fitness vectors.

## Takeaway

Removing the double count only works if the admission threshold is retuned with it. Doing both together reaches parity with the parent, but it does not beat it. The double-count and `>= 2` design is therefore roughly equivalent to single-count with `>= 1`. The simpler single-count form is a cleaner base, but admission-rule tuning of this kind is not where further hit-rate gains are likely to come from.
