# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Storing 64-bit fingerprints in the ghost queue left the single measured dimension (hit-rate) exactly where the parent had it, which fits the hypothesis's prediction that admission decisions would be unchanged. The ghost-side memory saving and any per-op gain from integer comparison are not in this fitness vector, so they remain unverified. The change also adds a second key hash on every ghost check and ghost push, so a constant-factor speedup on the miss path is not guaranteed; the linear `ghost` scan and `retain` are still present, so the asymptotic cost is untouched.
