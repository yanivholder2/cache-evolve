# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Giving each demoted-from-protected key one bounded extra probationary lap (the
`reprieve` flag + `prob_reprieved_len` guard, with re-entry at the MRU end
instead of the parent's next-victim front) left hit-rate exactly unchanged on
workload-a. The reprieve only fires when `enforce_protected_budget` actually
demotes a node, and at the 95/5 split the ~1945-slot protected segment rarely
overflows — so reprieves are seldom minted, and the few demoted keys that earn
one are cold enough that one more pass through the ~103-slot probationary window
still wins them no second touch before eviction. Spending residency on a single
extra lap targets the same warm-but-evicted band the ghost variants found too
thin, and finds it equally uneconomical: the binding constraint stays the
probationary floor's *size*, not how long a demoted key is allowed to linger in
it.
