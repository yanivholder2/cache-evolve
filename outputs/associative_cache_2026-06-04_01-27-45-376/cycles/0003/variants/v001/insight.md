# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The per-hit decay of every other slot's `lfu_counter` drains protection from items that were genuinely re-referenced — any slot whose hits are sparser than the bucket's overall hit rate decays back to 0 and re-enters the untouched-first eviction filter, so re-used items get evicted as if they were one-shot churn.

## Takeaway

Turning `lfu_counter` from a lifetime hit-count into a "recent-relative" counter via on-hit decay collapses the two-tier protection v003 relied on: under decay there is no stable "survived ≥1 hit" class, only "hit more frequently than the bucket's mixing rate," which on this workload is a higher bar than re-referenced items can clear. The aging step is not a free strengthening of v003's filter — it converts the filter's binary survivor/non-survivor split into a frequency race that the untouched-first scan was never designed to read.
