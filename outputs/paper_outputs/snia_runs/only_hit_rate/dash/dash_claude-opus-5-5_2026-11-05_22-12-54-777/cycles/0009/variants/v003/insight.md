# Variant v003 Insight

> Note: I could not list the overlay directory with the tools I had, so I did not inspect v003's source. The reading below rests on the H03 hypothesis and on both fitness vectors. It does not check how the change was implemented.

## Helped

(none: no cell improved on v002)

## Hurt

- hit-rate: web_0 at both sizes and hm_0 at 131072 each dropped by the smallest step the vector reports. hm_0 at 16384 did not move. These shifts are within noise, so the per-segment `bypass_score` changed almost nothing about which items stayed in target. In particular, the hypothesis aimed to recover cycle 0007 v002's gain on web_0 at 131072 by letting returners bypass all-warm windows, and that recovery did not happen.

## Takeaway

Adding a third gate branch, "bypass when `bypass_score > 0`," on top of v002's free-slot and counter-0 rule had no measurable effect. On web_0 at 131072, the returners that the cold-slot gate admits are evidently not re-hit in target often enough, before eviction or demotion, to keep the score positive. Or the all-warm-window case is too rare for the extra branch to matter. Either way, v002's web_0 large-size loss does not come from blocking returners whose value a re-hit feedback signal could detect. The next attempt should check how often each gate branch fires and how the score is distributed per segment before building on this signal. If the score sits mostly at or below zero, a re-hit-before-eviction counter is too pessimistic a sign of a returner's value under this demotion cascade. Tracking whether a returner is ever re-hit, including from the stash, may separate useful returners better.
