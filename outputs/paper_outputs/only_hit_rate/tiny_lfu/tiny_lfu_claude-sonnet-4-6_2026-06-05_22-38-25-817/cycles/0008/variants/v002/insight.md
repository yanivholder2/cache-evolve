# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Periodic count halving at the `width * 8` interval (131 072 ops at capacity=2048) erased accumulated frequency signal faster than stale counts accumulated on this trace, causing the sketch to underestimate hot-item frequencies and degrade both the PROMO_THRESHOLD gate and the admission gate accuracy.

## Takeaway

Even at the very slow `width * 8` decay cadence — 8× slower than canonical CMS halving recommendations — periodic right-shifting marginally reduced hit rate on workload-a. This indicates the trace does not exhibit meaningful count inflation at this timescale: the 8× sketch width introduced in v001 already keeps collision-noise low enough that the valid frequency signal lost to each halving costs more than the stale-signal noise it removes. Sketch aging is not a productive direction for this workload without evidence of count saturation or a longer-horizon working-set shift.
