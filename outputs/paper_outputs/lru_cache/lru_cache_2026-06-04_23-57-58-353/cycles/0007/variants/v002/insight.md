# Variant v002 Insight

## Helped

- hit-rate: SIEVE's visited-bit mechanism gives each item a second chance before eviction without reordering the list on `get`, which protects items that see even one reuse before the hand reaches them — a better fit for workload-a than strict recency-order eviction from the LRU head.

## Hurt

## Takeaway

Replacing LRU's promote-on-access discipline with SIEVE's in-place visited-bit scan lifted hit rate on workload-a. The key difference is that SIEVE separates "was this item accessed?" from "where does it sit in insertion order," letting the hand skip over any item that received at least one hit regardless of its age, whereas LRU would have already promoted that item to the tail (wasting a list operation) or allowed it to age out. This result suggests workload-a contains a meaningful fraction of items that receive a single reuse well after insertion — items that benefit from a second-chance pass rather than immediate eviction.
