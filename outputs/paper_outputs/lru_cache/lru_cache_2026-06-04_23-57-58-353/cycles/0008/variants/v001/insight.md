# Variant v001 Insight

## Helped

_(none)_

## Hurt

- hit-rate: Narrowing the probationary scan from the LRU half to the LRU quarter caused a small but real regression, showing that IRR values within the half-window are not uniformly MAX — the half scan was occasionally skipping past the very-head cluster and finding a slightly colder victim deeper in the list.

## Takeaway

The hypothesis that IRR=MAX cold newcomers cluster tightly enough at the LRU head to make a quarter-width scan sufficient was not borne out on workload-a: the half-scan in the parent was finding marginally better eviction victims in the second quarter of the probationary list, and halving the window discarded that signal. The scan width in `evict_from_probationary` is not purely overhead on an IRR baseline — it carries selection quality, so reducing it below half costs hit rate even when the eviction signal is IRR.
