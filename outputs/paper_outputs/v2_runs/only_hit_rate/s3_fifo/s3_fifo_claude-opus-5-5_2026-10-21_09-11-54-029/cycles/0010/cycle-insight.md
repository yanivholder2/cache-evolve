# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009's v003 (clear-on-pass SIEVE on 32-bit identity-hashed fingerprints). Each changed one structural parameter. v001 re-admitted ghost hits just ahead of the hand instead of at the head. v002 cut the small queue from 5% to 2.5%, which also grew main and the ghost. v003 halved the ghost ring. v002 won with only a marginal hit-rate gain, measured on one trace at one size. v001 and v003 both regressed.

## Patterns

- Two of three variants reopened levers that earlier Continuations had marked closed: partition and ghost capacity. The partition result went against that guidance, but only slightly. Under SIEVE the split is near flat, and its optimum may sit a little below the FIFO-era 5%. The ghost-capacity result agreed with it.
- The ghost mechanism is now bracketed from all sides. A larger ghost, a smaller ghost, more re-admit protection and less re-admit protection have all lost. Ghost hits on workload-a are genuine long-reuse signals, at a reuse distance of about one main length. Re-admitted entries need about one full sweep of residency, and a ghost of about main_size entries is what catches them. Head insertion at freq 0 with a main-sized ring is a local optimum. Cycle 0009's open question about giving ghost re-admits less protection is answered: no.
- v002's gain is confounded. Main and ghost grew together, so the variant cannot show whether the gain came from extra main residency or from extra ghost reach. Given v003's result, ghost reach is a plausible contributor.
- No variant addressed the measurement gap. Per-op-time, metadata-size and generalisation to another trace or size are still unmeasured, now for the tenth cycle. v003's metadata saving, for example, could not be credited.

## Open questions

- Does the small-queue curve keep rising below 2.5%, or does it turn down?
- Is v002's gain from main space or from ghost reach? Testing this needs a ghost budget fixed independently of main.
- Would changing which keys get ghosted help, for example main victims only versus small rejects? That changes ghost contents, not ghost size or placement.

## Continuation

Use v002 (2.5% small queue, ghost ring sized to main) as the baseline. Drop ghost re-admission position and ghost ring length, since both are settled. If policy budget remains, run one probe further down the split (about 1.25%), and optionally one that holds ghost size fixed while main grows, to separate the two effects. Expect small effects in either case. The real priority remains adding per-op-time, metadata-size and a second trace or cache size. Single-trace hit-rate tuning is at noise level, and further hypotheses cannot be judged without those measurements.
