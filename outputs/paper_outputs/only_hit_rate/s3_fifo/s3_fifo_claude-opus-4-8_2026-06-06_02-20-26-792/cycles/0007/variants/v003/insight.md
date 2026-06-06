# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Re-admitting main-evicted ghosts at `freq = 1` (via the new `from_main` tag and `Entry::with_freq`) granted one extra `evict_main` pass of protection, but on this LIRS trace that protection mostly shielded keys that were never reaccessed — costing a slot that a stronger resident could have held — so hit-rate slipped a hair below the parent's freq-0 re-admission.

## Takeaway

Warm-starting re-admission at `freq = 1` for the *strong* main-evicted population fails the same way it failed for the weak small-evicted one in cycle 0004: a one-pass survival credit only pays off when the protected key is actually reaccessed inside that window, and here too few re-admitted main victims are. Origin-aware ghosting (cycle 0006) was a real lever; layering eviction-protection on top of it is not — the credit is wasted slack, not recovered reuse.
