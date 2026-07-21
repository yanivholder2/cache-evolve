# Variant v001 Insight

## Helped

- `hit-rate`: Doubling the best-of-2 probe rate from 1-in-8 to 1-in-4 (mask `& 3` instead of `& 7`) rescued a small additional share of frequency-blind evictions, producing a measurable — if modest — lift over the parent's already-improved baseline.

## Hurt

- `per-op-time`: The extra sketch `estimate` calls now occurring on every 4th eviction instead of every 8th pushed put p50 from 100 ns to 200 ns (and p99 from 300 ns to 500 ns), eliminating the latency advantage the parent held over get and breaking the per-op-time parity the hypothesis aimed to preserve.

## Takeaway

Increasing the best-of-2 probe frequency from 12.5 % to 25 % does yield a small additional hit-rate gain, but it also makes the two extra sketch `estimate` calls visible in p50 latency — meaning the latency-neutral zone for this probe strategy lies between 1-in-8 and 1-in-4, and the sweet spot has been bracketed from above. Any further hit-rate improvement along this axis requires either accepting higher put latency or finding a way to amortise the sketch lookups more cheaply (e.g., reusing already-computed estimates rather than issuing fresh calls per eviction).
