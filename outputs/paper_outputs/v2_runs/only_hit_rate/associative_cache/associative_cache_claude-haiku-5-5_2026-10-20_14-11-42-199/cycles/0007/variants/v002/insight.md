# Variant v002 Insight

## Helped

- hit-rate: The second-miss ghost list admits a key only when it misses a second time within the FIFO window, so keys that are touched once no longer displace resident entries on workload-a. That is the mechanism behind the gain over baseline, consistent with the hypothesis's claim that rejecting one-shot inserts is the lever.

## Hurt

## Takeaway

On workload-a, gating admission on recurrence (exact-key ghost list, no frequency counter) is a productive lever distinct from the frequency-comparison gate tried in cycles 0004 to 0006: it works by refusing first-time inserts rather than comparing estimated frequency against the victim. The cost side is unresolved for this variant: its fitness vector reports only hit-rate, with no metadata-size or per-op-time entries, so the ghost list's memory and latency overhead the hypothesis flagged remains unmeasured.
