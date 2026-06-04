# Variant v002 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: The ghost grants a `lfu_counter = 1` head start to keys that completed the full primary→stash→ghost journey without a single re-access, marking them as mild eviction resistors at the moment they are re-inserted — but those are precisely the keys the workload has already signaled are cold, so the ghost boost misidentifies cold items as warm ones and displaces better candidates from primary.

## Takeaway

A ghost tier's value depends entirely on whether the keys it remembers are likely to be re-used. Keys that aged through both primary and stash without ever being accessed represent the lowest-reuse cohort in the workload; giving them a frequency credit on re-entry inverts the LFU signal and makes the eviction order slightly worse. The stash already captures the useful second-chance signal for items with demonstrated re-access potential; extending memory one tier further to items with zero demonstrated re-access adds noise rather than coverage.
