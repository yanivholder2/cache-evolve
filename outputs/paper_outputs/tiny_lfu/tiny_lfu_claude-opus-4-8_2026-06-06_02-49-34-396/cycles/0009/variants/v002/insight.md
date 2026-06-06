# Variant v002 Insight

## Helped

- hit-rate: Splitting the single LRU into probation+protected segments and drawing victims only from probation does shield twice-proven entries from one-hit newcomers, nudging the saturated hit-rate up — but only marginally, so the SLRU lever barely moved a metric the admission gate was already filtering.

## Hurt

- metadata-size: The new `protected: bool` flag on `Node` pushes the struct past its 8-byte alignment boundary (24→32 bytes per entry), and the four added cache fields (`prot_head`, `prot_tail`, `protected_size`, `protected_cap`) grow `TinyLfuCache` from 128 to 152 bytes.
- per-op-time: Every `get` hit now runs `record_access` — an unlink plus `push_back_protected` plus a conditional protected-LRU demotion — instead of the parent's single unlink+push_back, raising `get` p50/p99; `put` p50 also rises from the added probation/protected bookkeeping on admission.

## Takeaway

Restructuring the main cache into a two-segment SLRU is a real eviction lever, but on this workload the existing `incoming_freq > victim_freq` admission gate already blocks most one-hit newcomers, so the segment split contributes only a sliver of extra hit-rate while charging a per-entry footprint byte (the alignment-inflated `protected` flag) and a promotion/demotion cost on every hit. The hit-rate ceiling here is set by admission, not by list structure.
