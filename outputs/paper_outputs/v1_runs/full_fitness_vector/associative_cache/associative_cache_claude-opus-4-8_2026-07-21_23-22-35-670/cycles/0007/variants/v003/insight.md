# Variant v003 Insight

## Helped

- metadata-size: The cap change touches only the `protected_cap()` off-`Item` knob, so the per-item floor stays exactly at key+value and metadata held.

## Hurt

- hit-rate: Reserving only one probationary slot (`max_size - 1`) collapses the two-segment ordering toward a plain LRU list — with almost no probationary zone the bucket loses the scan-resistance that segmentation buys, so hit-rate did not recover and drifted marginally below the parent.
- per-op-time: A near-total protected segment means protected hits rotate over a much longer span (`items[position..].rotate_left(1)` now spans most of the bucket), copying more bytes per access than the `max_size/2` cap did, and per-op p50/p99 rose across both get and put.

## Takeaway

Pushing `protected_cap()` to `max_size - 1` did not turn cycle-6's aggressive-cap regression into a gain: with a single probationary slot the SLRU degenerates into LRU, so hit-rate stayed flat while the longer protected-MRU rotations slowed every access. The optimal cap sits neither at `max_size/2` nor near `max_size` — near-total protection is the wrong end of the knob.
