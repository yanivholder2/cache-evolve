# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The adaptive threshold machinery occasionally shifted `evict_thresh` away from `1` during a workload that has no meaningful phase transitions, causing brief periods where eviction was either more or less aggressive than the static-`1` optimum and marginally reducing the resident set quality.

## Takeaway

On workload-a — a single, phase-stable trace evaluated at the same size as `ADAPT_PERIOD` — the main-hit ratio stays in a narrow band that keeps `evict_thresh` at `1` almost throughout, so v003 effectively runs as v001 with extra accounting overhead. The one instance where adaptation fired with a shifted threshold appears to have hurt slightly rather than helped, suggesting that workload-a's optimal eviction point is `freq > 1` everywhere and the dynamic mechanism has no phase signal to act on. The hypothesis needs a workload with detectable scan bursts or hot-set transitions to test whether the three-level control surface actually self-tunes usefully.
