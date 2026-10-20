# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Cutting `protected_cap` from `(capacity*4)/5` to `(capacity*3)/5` shrank the protected tier that holds twice-touched keys; with fewer protected slots, promoted reused keys get demoted and evicted sooner, so the LIRS trace's reuse stream lands fewer hits.

## Takeaway

Widening probationary from 20% to 40% did not buy more promotion runway — it refuted the hypothesis. On this trace the binding constraint is protected-tier residency, not probationary dwell time: a key's second touch already arrives within the 20% window, so the extra probationary room was wasted while the stolen protected capacity directly cost retained reused keys. The 80/20 split was better-tuned than 60/40; the ratio should move the other way, not this one.
