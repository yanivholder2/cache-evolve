# Variant v003 Insight

## Helped

- hit-rate: Inserting new entries mid-bucket and promoting hits one step by transpose means one-touch entries (scan traffic) sit closer to the LRU end and are evicted first. Entries that get re-referenced climb gradually and are not flushed by a burst of cold keys, which plain move-to-front allows.

## Hurt

- per-op-time: Put p99 rose, and get p50 rose slightly. Mid-bucket swap placement and the transpose on hit add data-dependent element shuffling, which makes put tail latency more variable than the baseline's simple remove+push.

## Takeaway

A stateless change to insertion position and promotion step (no counters, no extra bits, so metadata-size is unchanged) is enough to get scan resistance in a small associative bucket. The cost is a modest, mostly tail-latency, penalty on put. Note that the insight was written from the hypothesis and fitness vectors only. I could not inspect the source diff in this session.
