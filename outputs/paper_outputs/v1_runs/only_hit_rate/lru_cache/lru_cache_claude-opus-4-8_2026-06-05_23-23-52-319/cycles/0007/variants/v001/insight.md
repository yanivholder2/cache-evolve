# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Splitting the flat protected LRU into cold/hot tiers with cold-first demotion (`enforce_protected_budget` evicts `cold_head` before `hot_head`) made the cold list a thin, fast-cycling buffer: as the hot list accumulates, `cold_len` is squeezed, so each new 2nd-access promotion immediately pushes the region over budget and demotes the cold LRU straight back to probationary — churning out mid-reuse keys before they can earn a 3rd access, which the parent's flat protected segment would have retained for a full protected-length of activity.

## Takeaway

Ranking protected residency by recency-of-Nth-access backfired on this trace: requiring three hits to reach the safe (hot) tier while giving cold keys only a hot-starved window meant the split shed exactly the twice-seen working set the hypothesis hoped to keep. Within the converged 95/5 geometry, a single flat protected LRU retains the reuse-heavy set better than a depth-stratified one — reuse depth is not the binding signal here.
