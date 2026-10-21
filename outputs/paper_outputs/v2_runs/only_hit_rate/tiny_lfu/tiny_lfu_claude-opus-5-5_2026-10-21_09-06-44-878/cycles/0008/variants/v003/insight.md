# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The idle discount (`est >> min(idle / capacity, 8)`) is applied only to the sampled residents in `select_victim`, while the newcomer is scored on its raw sketch estimate. This makes the strict `incoming_freq > victim_freq` gate much easier to pass. The 8 oldest probation entries are by construction the least recently touched, so on a 2048-entry cache with a tick on every get and put they are usually at least one `capacity` idle and lose one or more halvings. Once an entry is idle for 8×capacity its effective value is 0, and any newcomer with an estimate of 1 or more evicts it. Moderately frequent residents with long reuse distances, the keys lifetime counts were protecting, now lose to cold one-off arrivals. Admission drifts toward LRU-like behaviour and the scan resistance TinyLFU gave the parent is lost.
- hit-rate: `demote_stale_protected_head` adds a steady leak from protected to probation. On any full-cache put where the protected LRU has been idle for more than `2 * capacity` ticks, it is pushed to the probation **head**. It then enters the victim sample already carrying at least two halvings. Keys re-referenced at intervals just beyond 2×capacity, which LIRS-style traces like workload-a contain, are exactly what the parent's protected segment kept resident. Here they get demoted and evicted before their next reuse.

## Takeaway

Recency-discounting only the victim side of the TinyLFU contest is not a narrow fix for "stale-high" protected keys. It weakens admission everywhere, because the probation head is always the most idle part of the cache. This variant lands both changes at once, so it cannot show how much of the loss each one causes. The asymmetric discount is the broader mechanism, because it acts on every eviction, while demotion acts on at most one entry per put. A follow-up should either discount both sides of the comparison on the same scale, or test protected-head demotion with raw estimates and no discount. That would show whether demotion by itself helps or hurts. The parent's lesson still holds: on workload-a, lifetime frequency in the sampled contest is the asset, and per-entry idle decay erodes it faster than it removes stale residents.
