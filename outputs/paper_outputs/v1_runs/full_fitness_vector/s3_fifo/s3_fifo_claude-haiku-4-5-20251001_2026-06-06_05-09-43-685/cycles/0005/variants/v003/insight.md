# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: Preserving frequency across ghost entries and re-admitting with preserved_freq + 1 breaks the hit-rate signal; items evicted from main without frequency decay enter ghost with inflated hotness, causing less accurate readmission decisions and reducing overall cache effectiveness.
- per-op-time: Ghost queue lookups in `put()` to extract frequency (lines 85) add latency; changed `evict_main()` semantics (moving to ghost instead of in-place frequency decrement) further increases put operation cost from 577ns to 687ns (p50) and 1462ns to 3404ns (p99).

## Takeaway

Frequency preservation in ghost without decay creates more harm than benefit. The hypothesis attempted to split the difference between v001's aggressive decay (which hurt hit-rate) and v003's structural overhead, but instead introduced both slower latency (from ghost frequency lookups) and worse hit-rate (from incorrect frequency semantics). The core issue: moving hot items directly to ghost with full frequency intact violates the admission principle—items evicted from main queue should graduate with weakened rather than preserved signals, or they corrupt the distinction between hot and cold candidates. This variant teaches that ghost frequency tracking requires careful decay or reset semantics to remain a valid filter.
