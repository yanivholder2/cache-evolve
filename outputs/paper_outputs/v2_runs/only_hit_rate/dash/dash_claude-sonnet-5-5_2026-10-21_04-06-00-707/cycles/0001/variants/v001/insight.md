# Variant v001 Insight

## Helped

- hit-rate: `DashSegment::get` now promotes with `InsertPosition::Back`. Stash hits, probing retries and cascades to stash land at the MRU end of the target bucket. Items that just proved useful are no longer the next eviction victim, which suits the recency-skewed LIRS trace.

## Hurt

- No dimension measurably regressed. The fitness vector reports only hit-rate, so metadata-size and per-op-time were not evaluated. The change only swaps an enum argument, so no new metadata is expected. I could not read the overlay source or diff it against the parent, so this rests on the hypothesis text and the fitness vectors.

## Takeaway

Where a promoted item lands in its bucket is a cheap, one-argument lever that moved hit-rate on workload-a. Baseline's cold (LRU-end) insertion of freshly hit items was leaving hits on the table. Whether MRU placement holds up on other traces, and whether it costs anything in per-op time, remains untested because only hit-rate was measured.
