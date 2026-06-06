# Variant v003 Insight

## Helped

*(no dimensions improved)*

## Hurt

- hit-rate: Doubling the ghost capacity extended the tracking window to 3072 keys, causing ghost hits from more-distant evictions to promote items directly into main — bypassing the small queue's frequency filter — and polluting main with items whose re-access was incidental rather than indicative of genuine re-use.

## Takeaway

A larger ghost window does capture more re-accesses, but for workload-a the additional hits are from keys evicted long ago whose single re-appearance does not signal sustained demand. Because ghost hits fast-path items straight into main without the frequency threshold that small-queue graduation enforces, extending the window injects cold items into main and displaces entries that earned their residence through repeated access. For this trace, the ghost queue's value comes from recapturing truly recent evictions, not from accumulating a longer history.
