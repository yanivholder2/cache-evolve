# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Halving the protected segment (from 4/5 to 1/2 of capacity) gave probation more room, but on workload-a at size 2048 that cost protected entries that had already been re-referenced, so the incumbent's larger protected share was the better setting for this trace.

## Takeaway

Moving only the protected-segment cap shows the probation/protected ratio matters, but the 80/20 split chosen in v002 is at or near the better side for this workload: giving probation more room did not catch enough one-touch-then-return keys to offset the protected entries it displaced. Because the drop is small and comes from a single trace at one cache size, the split ratio looks like a second-order lever, and any further tuning should be checked on other workloads before the 4/5 default is changed.
