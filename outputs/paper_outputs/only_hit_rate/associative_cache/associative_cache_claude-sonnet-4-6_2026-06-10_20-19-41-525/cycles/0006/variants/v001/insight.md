# Variant v001 Insight

## Helped

## Hurt

- hit-rate: CLOCK's per-sweep decrements progressively convert warm items (counter > 0) into cold ones across successive eviction calls, causing the hand to eventually evict items that were still being accessed — items that the cold-first scan would have protected until a bulk-decay was actually needed.

## Takeaway

Replacing cold-first-scan + bulk-decay with CLOCK eviction hurt hit-rate. The cold-first scan only evicts items whose counter is genuinely zero (never accessed, or fully decayed by a bulk pass), whereas CLOCK silently drains individual counters on every eviction sweep, turning moderately warm items cold over several calls without any access drought having occurred. On this workload the bulk-decay fallback fires rarely enough that its counter-compression cost is low, while CLOCK's incremental aging incurs a steady cost by making still-relevant items eligible for eviction sooner than the cold-first policy would. Distributing aging pressure per-item is not free: it erodes the frequency signal that the counter cap was preserving.
