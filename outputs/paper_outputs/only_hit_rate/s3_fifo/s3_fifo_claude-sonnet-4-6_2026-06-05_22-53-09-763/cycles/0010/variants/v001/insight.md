# Variant v001 Insight

## Helped

- hit-rate: Adding unconditional move-to-front on main-queue hits — on top of the already-tight 5% small queue — pushed hot residents back to the front of main on each access, reducing the chance they would be swept out by `evict_main()` before the next reference arrived.

## Hurt

## Takeaway

MTF on main-queue hits compounds additively with the `capacity / 20` cold-admission filter: the two changes target distinct failure modes (positional drift of hot residents vs. one-time-access pollution of the admission queue), and stacking them yields a further hit-rate gain beyond what either mechanism delivers alone. The improvement is small relative to the H03 small-queue gain, indicating that on this Zipfian workload most hot keys are accessed often enough that positional drift was already a secondary bottleneck — but it is real and directionally confirms the combination is worth retaining as the new baseline.
