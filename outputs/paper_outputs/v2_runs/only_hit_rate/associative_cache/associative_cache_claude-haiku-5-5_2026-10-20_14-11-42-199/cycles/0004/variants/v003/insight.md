# Variant v003 Insight

## Helped

- hit-rate: The hypothesis adds a single shared frequency counter and admits a new key on a miss only when its count is strictly greater than the victim's, so one-shot inserts no longer displace reused entries on workload-a.

## Hurt

## Takeaway

Gating eviction-time admission on one frequency estimate shared by incoming and resident keys raised hit-rate over baseline on workload-a, which supports the hypothesis that the earlier cycle-0002 admission loss came from comparing estimates on different scales rather than from admission itself. This insight is drawn from the hypothesis and the fitness vectors: the variant's source overlay could not be read in this run, and the v003 fitness vector reports only hit-rate, so metadata-size and per-op-time are unmeasured here and no claim is made about them.
