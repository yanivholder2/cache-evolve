# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Periodic floor-halving (`*v /= 2` every `W = 10 * capacity` accesses) maps every frequency of 1 to 0, so one-hit keys already resident lose their weight. The admission gate uses `incoming.max(1)` against the victim's raw count, so a fresh one-shot key (floored to 1) now strictly beats a resident one-hit victim (0) and gets admitted, which lets one-shot inserts displace reused entries.

## Takeaway

Decaying the shared counters without removing keys still hurts, so the parent's zero-pruning was not the sole cause of the earlier aging losses. Halving is not neutral at small magnitudes: with floor division, the gate's distinction between one-hit and no-hit history disappears, and the `max(1)` floor on incoming keys then tips the strict comparison toward admission. Any follow-up aging should keep counts at or above 1 for previously seen keys (for example, `max(1, v / 2)` for nonzero `v`) so that one-hit history survives the decay.
