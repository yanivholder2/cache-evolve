## Recurring wins

- Capacity/ratio changes beat rule-level tweaks: shrinking the small queue won in cycles 2 and 3, but gains shrank each step and are now near-saturated.
- Ghost-side information and strictness helped: ghosting main victims (cycle 1) and a shorter ghost (cycles 3-4) won, but only marginally. The ghost-length and small-ratio gains are not additive.
- Each cycle's winner was a single small change built on the prior winner, and every gain was small.

## Recurring failures

- Loosening admission hurt in every cycle it was tried: a looser promotion gate, freq=1 on ghost hits, and a longer ghost. Weakly-reused keys pollute main.
- Faster ageing or decay of main entries lost every time (freq halving, lower freq cap). This workload rewards keeping reuse history.
- Filtering ghost hits was clearly worse (cycle 4). A ghost hit is strong reuse evidence, so keep that path permissive.
- A stricter promotion gate was a tie, so it is either non-binding or a no-op diff.
- Rule-level single-branch tweaks rarely pay off. Admission sizing is now saturated.

## Open questions

- Are the small gains real or noise? All evidence comes from one trace (LIRS) at one cache size. Test a second trace and cache size.
- Does the very small probation window hurt scan-heavy or other traces?
- Is S3-FIFO at a local optimum here? Untried structural levers: main-eviction policy, reinsertion rules, and loosening the path for ghost hits only.
- Did the stricter-gate diff change behaviour at all? Verify that future diffs alter behaviour.
- Metadata-size and per-op-time have been missing from the fitness vectors for four cycles. Can they be restored so cost can be judged?

## Per-policy notes

- S3-FIFO (the only policy so far): the current base is main-victim ghosting, a small queue much smaller than the original, and a shortened ghost. Strict filtering plus permissive ghost-hit re-admission is the pattern that works.
- Stop tuning the small ratio, ghost length, promotion threshold, and freq decay or cap. Move to structural changes and robustness checks.
