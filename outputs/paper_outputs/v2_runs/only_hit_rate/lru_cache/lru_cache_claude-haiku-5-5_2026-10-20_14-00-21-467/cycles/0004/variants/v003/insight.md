# Variant v003 Insight

## Helped

- hit-rate: Splitting the recency list into probation and protected segments, and evicting from probation first, keeps one-touch keys from displacing entries that were hit at least once, which is the mechanism H03 targeted on workload-a at size 2048.

## Hurt

## Takeaway

Segmenting the baseline recency list and preferring probation victims is a change that moves hit-rate on the scan-heavy workload-a trace without adding a ghost list or admission filter. This variant's fitness vector reports only hit-rate, so it says nothing about metadata size or per-op time; the segmentation adds a second list pointer and demotion logic, and those costs need a measurement before this result can be weighed against the baseline's cost profile.
