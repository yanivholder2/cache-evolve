## Recurring wins

- Only one cycle so far; nothing has repeated yet. Tentative signal: adding information (e.g. ghost coverage of main-queue victims) without changing admission or protection rules is safe, though the gain was marginal.

## Recurring failures

- Making main more permissive (looser small-to-main promotion, pre-protecting ghost re-admits with initial freq) hurt. It added pollution that displaced useful residents. Baseline strict filtering looks well tuned for the LIRS workload.
- Single-branch, single-condition tweaks only. None explored structural levers, and none gave large gains.
- Metadata-size and per-op-time were missing from the fitness vector, so cost claims are unverified.

## Open questions

- Can metadata and per-op-time be restored in fitness reporting, so ghost-size costs can be judged?
- Would stricter promotion, a different small/main split, or a different ghost capacity help, given that looser gating hurt?
- Does main-eviction policy offer headroom, since main-victim ghosting was only marginally useful?
- Is the main-victim ghosting gain stable across traces, or is it noise on LIRS?
- Do combinations of the ghosting change with structural levers compound?

## Per-policy notes

- S3-FIFO: Baseline admission is strict and well tuned on LIRS. Loosening it is a dead end. Main-victim ghosting is the current best base. Next probes: small/main ratio, ghost capacity, stricter promotion, main-eviction policy.
