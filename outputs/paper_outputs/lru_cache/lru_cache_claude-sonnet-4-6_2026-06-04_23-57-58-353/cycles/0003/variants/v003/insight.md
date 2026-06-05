# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Adding the second-chance bit required freezing protected nodes in place on access (no `unlink_prot`/`push_back_prot`), which decoupled list position from recency — so the LRU head within protected no longer reliably identifies the least-recently-used entry, and frequently-accessed entries can still sit near the head waiting for a demotion event to trigger their single reprieve.

## Takeaway

Disabling position updates on protected hits in exchange for a deferred single reprieve trades a continuous recency signal for an infrequent one: a protected entry only escapes demotion pressure once per demotion event, whereas the parent's move-to-tail on every hit keeps hot entries at the MRU end at all times. On this workload the ghost set already protects warm cross-residency keys at admission time, so the benefit of the second-chance reprieve is small while the cost of degraded in-segment ordering is concrete and persistent.
