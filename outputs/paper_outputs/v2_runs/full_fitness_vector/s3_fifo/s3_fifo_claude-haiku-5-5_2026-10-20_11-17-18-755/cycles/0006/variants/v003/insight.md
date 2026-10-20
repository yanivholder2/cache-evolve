# Variant v003 Insight

## Helped

- per-op-time: Both get and put tail and median latencies fell relative to the parent, consistent with the only code change (the small-queue share constant) shifting how often operations take the small-queue path versus the main-queue rotation path. The effect is modest and comes from a single-run measurement, so it may be partly noise.

## Hurt

## Takeaway

Raising the small-queue share to 20% on its own left hit rate on workload-a at 2048 exactly where the baseline was, so the hit-rate gain that cycle 0002 saw came from the victim-choice change bundled with it, not from the share increase. Metadata footprint is unchanged because `Entry` and the container layout were not touched.
