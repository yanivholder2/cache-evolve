# Variant v002 Insight

## Helped

- hit-rate: hm_0 at the small size was the only cell that moved up. A likely reason is that conservative update keeps the counts of cold, collision-prone keys closer to their true value. Probation residents with only a few real references then stop losing the contest to window candidates whose plain-update estimate was inflated by sharing a column with a hot key. That makes incumbents a little stickier, which suits hm_0 when the cache is small. hm_0 at the large size did not move in any meaningful way.

## Hurt

- hit-rate: web_0 fell at both sizes, which is the opposite of what the hypothesis predicted for its target trace. The sketch is already wide (`capacity*4`, power of two, depth 4), so collisions that inflate the *minimum* across 4 rows were rare to begin with. Conservative update therefore removed little noise that matters for decisions. What it mostly did was squeeze small counts into a narrow band of exact, equal integers. With the strict `cand_freq > victim_freq` admission and the double counting on miss-then-`put`, a newcomer that has been seen a couple of times now ties a coldest-of-4 victim more often, and a tie means the newcomer is rejected. Under plain update, small random inflation used to break those ties in the newcomer's favour some of the time. The change quietly raised the admission bar, and that undid part of the "lower the bar" effect that gave the parent (v003) its web_0 gain.

## Takeaway

At this sketch width, the error that matters in the admission contest is not collision inflation but ties at the low end. Conservative update makes those ties more frequent, and with strict `>` it lets fewer warm newcomers in, which costs web_0. The parent's web_0 gain came from candidates winning more often. Any future sketch-precision change should be paired with an explicit tie-break that favours the candidate (for example `>=`, or a recency-based tie-break). Without it, a "more accurate" sketch acts as a tighter admission filter. Note too that the sketch never ages (u32 counters, no cap, no reset), so cutting inflation alone does nothing about stale frequency history.
