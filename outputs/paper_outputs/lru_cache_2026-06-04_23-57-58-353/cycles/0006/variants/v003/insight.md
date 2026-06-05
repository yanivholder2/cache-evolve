# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The decay's half-life of 512 operations is too short relative to the access pattern in workload-a — entries that carry legitimate frequency from protected residency sit idle in probationary longer than 512 ops between access bursts, causing `effective_freq` to collapse their signal to zero and the minimum-frequency scan to treat them the same as cold newcomers, producing more evictions of warm content.

## Takeaway

Logical-clock decay at `FREQ_HALF_LIFE=512` undermined the minimum-frequency scan rather than sharpening it: the scan's existing value comes from distinguishing demoted-from-protected entries (freq > 0) from cold newcomers (freq = 0), but with a half-life shorter than the typical inter-access gap for warm entries in this LIRS workload, the decay erases that distinction before victim selection ever happens. The result is a stricter form of the same problem the decay was meant to solve — cold and warm entries look identical — just with the threshold moved from "never decays" to "decays too fast."
