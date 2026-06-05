# Variant v001 Insight

## Helped

- hit-rate: Pushing `protected_cap` from `(capacity * 9) / 10` to `(capacity * 19) / 20` gives hot keys still more residency before demotion, and the 103-slot probationary segment stays non-empty so probationary-first eviction keeps draining one-hit-wonders — a marginal net gain over the parent on this reuse-heavy LIRS trace.

## Hurt

## Takeaway

At 95/5 the SLRU lever still pays, but only barely: shrinking probationary from 205 to 103 slots produced a far smaller hit-rate gain than each prior protected enlargement did, signalling the ratio is approaching the point where further protected room stops converting into hits. The change confirms the direction has not yet broken the eviction path, while flagging that the probationary floor — not appetite for protected tenure — is now the binding constraint.
