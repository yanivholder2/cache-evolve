## Recurring wins

- Adding information without changing admission/protection rules is safe (cycle 1 main-victim ghosting), though the gain is marginal.
- Structural capacity allocation beats rule tweaks: shrinking the small queue (more room for main) was the clearest gain (cycle 2), and it stacked on the cycle 1 winner.
- Building on the prior winner and changing one knob at a time has worked as a lineage (ghosting -> smaller small queue).

## Recurring failures

- Loosening admission or promotion lets weakly-reused keys into main and displaces useful residents. This failed with a looser gate, ghost-hit freq=1, and a longer ghost queue.
- Aging main faster (freq halving) hurt slightly. The linear decrement looks well tuned.
- Rule-level tweaks have lost in both cycles. Baseline filtering looks well tuned for this workload.
- Gains so far are modest, and every one was measured on a single trace (LIRS) at one cache size.

## Open questions

- Where is the optimum small-queue ratio, and does the gain keep growing as it shrinks? Sweep both sides of the current value.
- Does a smaller probation window hurt scan-heavy or other traces? Check robustness on a second trace and other cache sizes.
- Would a shorter ghost, or a stricter ghost-hit rule, help, given that a longer ghost hurt?
- Would a stricter promotion gate or a different main-eviction policy help?
- Metadata-size and per-op-time are still missing from every fitness vector, so cost claims are unverified. Why, and can they be restored?

## Per-policy notes

- S3-FIFO (the only policy so far): the small/main ratio is the most responsive lever. Ghost length acts as an admission filter, so longer means looser and worse. Main eviction with linear freq decrement is near a local optimum. The current best is baseline plus main-victim ghosting plus a smaller small queue.
