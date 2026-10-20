# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Periodic right-shifting of every count-min counter (`CountMinSketch::halve`, triggered from `record_access` every `capacity * 10` accesses) discards frequency history that the admission check `incoming_freq > victim_freq` depends on, so on workload-a at size 2048 fewer newcomers clear the LRU victim.

## Takeaway

Halving the sketch on a fixed 10×capacity schedule removes the stale-history problem the hypothesis assumed was blocking admissions: at this window size the decay mostly erases low-count signal, and integer truncation pushes many newcomers and victims to equal estimates, which the strict `>` rejects. Any future decay scheme should be gated on evidence of staleness, or use a gentler decay than a full right-shift.
