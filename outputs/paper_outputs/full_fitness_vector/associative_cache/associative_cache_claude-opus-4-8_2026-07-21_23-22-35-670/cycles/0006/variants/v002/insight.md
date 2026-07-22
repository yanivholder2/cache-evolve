# Variant v002 Insight

## Helped

- per-op-time: Turning a protected hit into a bare position return drops the `self.items[position..].rotate_left(1)` from the common hot-key path, so put's p50 falls back to the baseline single-tier cost while p99 holds.

## Hurt

- hit-rate: The hypothesis's "intra-protected order never affects eviction" invariant only holds while a probationary item exists at index 0; once a bucket is fully protected, index 0 is itself the protected victim, and skipping the MRU rotation lets a recently-hit hot key sit at the eviction end and get dropped — a small hit-rate regression against the parent.

## Takeaway

Lazy protected-hit reorder successfully sheds the per-hit segment rotation and recovers put p50, confirming that rotation is pure overhead in the mixed-segment case. But it is not hit-rate-neutral: when a bucket saturates to all-protected, eviction reads the intra-protected order the rotation used to maintain, so dropping it trades a hair of eviction quality for the latency win.
