# Cycle 0007 Insight

## Cohort outcome

All three variants came from cycle 6's v003 (`small` at a twentieth of capacity, ghost at an eighth). Each tested one follow-up that cycle 6 had asked for:
- **v002** shrank `small` to a fortieth and grew the ghost so that `small + ghost` stayed the same size. It won, by a marginal amount.
- **v001** ported `MAX_FREQ = 7` onto the parent. Its gain was marginal, so the two levers overlap rather than stack.
- **v003** started ghost-admitted keys at `freq = 1`. Hit rate went down slightly.

## Patterns

- **Holding the combined window constant still works, but the gains are shrinking.** Moving resident probation slots into the ghost helped again, by less than last cycle. This repeats the pattern seen with ghost-only halvings. Two forces work against the extra space in `main`. `small` now catches less reuse while keys are resident. More keys also reach `main` through the ghost with no credit.
- **Retention changes in `main` are close to used up.** A higher frequency cap adds little once `main` is large. The reason is that credits only build up while a key is in `main`, and both admission paths reset them to zero.
- **Starting credits have to respect how strong the evidence is.** A ghost hit shows two references, and none of them happened while the key was resident. Giving these keys a free credit ranked them above `small` promotions, which did show a hit while resident. `main` then held on to false positives for longer. With a short window, `main`'s CLOCK loop already protects keys that really are reused. This contradicts the parent's idea that ghost arrivals "lack protection".
- **The cohort pattern is the same as in earlier cycles: three small, single-knob moves, all measured on one trace at one cache size.** A second cache size, the fingerprint-ring port, and measurement of `metadata-size` and `per-op-time` are still missing, for the seventh cycle running.

## Open questions

- Does another shift from `small` to the ghost still help, or is `small` at a fortieth the floor? Does the split rule hold at a second cache size?
- Would starting credits that are consistent across both admission paths help? For example, give a matching credit to `small` promotions, or give a credit only on a repeat ghost hit.
- If credits carried over from `small` into `main` instead of resetting, would the higher frequency cap start to pay off?

## Continuation

Use this cycle's v002 as the parent, and keep `MAX_FREQ = 7` on it since it costs nothing. Before shifting any more slots from `small` to the ghost, run the current split at a second cache size. The admission-window line of work now produces gains too small to trust from a single size.

Drop raising the frequency cap again, and drop starting credits that apply to only one admission path. If credit work continues, change how credits are earned or carried over, and treat both admission paths the same way. Still pending: the fingerprint-ring port, and measuring `metadata-size` and `per-op-time`.
