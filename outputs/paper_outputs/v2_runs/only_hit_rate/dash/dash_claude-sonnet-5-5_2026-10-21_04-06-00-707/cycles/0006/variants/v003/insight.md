# Variant v003 Insight

## Helped

- hit-rate: Admitting a candidate only when its count-min estimate beats the would-be victim's keeps frequently used entries from being displaced by one-off keys. The gain over the baseline's blind admission is small.

## Hurt

- No dimension measurably regressed. The variant's fitness vector reports only hit-rate, so metadata-size and per-op-time are unmeasured. The added sketch (4 rows of 4-bit counters, about 4x capacity wide) and the increment on every get and insert should add metadata and per-op cost. Those costs are untested here.

## Takeaway

A frequency-comparison admission gate on its own, with no second-sighting filter, gives only a marginal hit-rate improvement on workload-a at this cache size. The baseline's LRU eviction already captures most of the available locality, so a TinyLFU-style gate adds little there. Whether the sketch's memory and per-op overhead is worth that gain can't be judged until those dimensions are evaluated. I couldn't read the overlay source in this session, so this interpretation rests on the hypothesis text and the fitness vectors.
