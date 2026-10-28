# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): This cell rose by a tiny amount, so the hypothesis's warning sign did not show up. That warning was that stale counts, saturated at 15 and only halved every 64 × capacity, would lock out newly hot keys on hm_0. At this size, holding counts across twice as many increments did not hurt hm_0's admission decisions.

## Hurt

- hit-rate (web_0 @ 16384): This cell slipped by a tiny amount, about the same size as hm_0's gain. That disproves the hypothesis's claim that 64 × would keep lifting web_0's multi-fill reuse. Whatever reuse-gap history 32 × left unprotected, doubling the period again did not recover it. A likely offsetting effect: the doorkeeper is cleared only on reset, and it has 4 probes into roughly 4 × capacity bits. Over a 64 × capacity period it fills up, so it stops absorbing first occurrences. One-shot keys then gain counts that let them tie or beat probation victims more often.

## Takeaway

Moving the small-size aging period from 32 × to 64 × capacity reached a plateau. The two 16384 cells moved by equal, opposite, noise-sized amounts, the summary did not change, and the 131072 cells matched the parent exactly as the size gate guarantees. So nearly all of the memory-length gain at ≤ 32_768 is already captured by 32 ×. Past that point, 4-bit saturation and doorkeeper fill seem to cancel any extra benefit, while the hypothesis's predicted failure (hm_0 lockout) never appeared. Further gains at small sizes probably need a different lever, such as doorkeeper sizing or reset policy, or counter resolution, rather than a still-longer period.
