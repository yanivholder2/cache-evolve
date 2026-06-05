# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Gating promotion on a second probationary hit (`PROMOTE_AT = 2` plus the per-`Node` `hits` counter) delays protection by one access, so a key that is reused once but evicted before its second hit never reaches the protected tier — the extra probationary residency does not save it, and hit-rate slips below the parent's promote-on-first-hit behavior.

## Takeaway

On this LIRS trace, raising the promotion threshold from one to two hits made the protected tier stricter but not better: many genuinely reused keys are touched a second time only after they have already churned out of the probationary LRU, so the harder signal loses hot keys instead of purifying the set. First-hit promotion (parent v001) extracts the reuse signal earlier and remains the stronger rule here.
