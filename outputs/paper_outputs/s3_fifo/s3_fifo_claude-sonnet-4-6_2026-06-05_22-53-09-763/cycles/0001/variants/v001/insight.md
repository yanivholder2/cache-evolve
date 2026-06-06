# Variant v001 Insight

## Helped

- hit-rate: Shrinking `small_size` from `capacity / 4` to `capacity / 10` freed ~307 additional resident slots in the main queue for a 2048-slot cache, giving the second-chance eviction policy more room to keep frequently-reaccessed items alive and reducing the rate at which popular entries are displaced.

## Hurt

## Takeaway

The small queue's role as a one-time-use admission filter does not require 25% of capacity: 10% is sufficient to absorb cold misses while handing the freed resident slots to the main queue, where second-chance promotion does the heavy lifting. On workload-a, whose LIRS trace exhibits temporal locality beyond the baseline main-queue footprint, the larger main queue directly absorbs the working set that previously spilled into eviction — validating the hypothesis that the 25% baseline over-allocates to the admission queue at the expense of hit rate.
