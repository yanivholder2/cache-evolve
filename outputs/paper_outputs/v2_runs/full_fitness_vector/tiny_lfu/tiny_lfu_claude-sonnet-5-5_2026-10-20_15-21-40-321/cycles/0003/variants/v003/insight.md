# Variant v003 Insight

## Helped

- metadata-size: unchanged. The edit only alters a comparison operator, so no struct, node or sketch layout moved.

## Hurt

- hit-rate: Admitting on `incoming_freq >= victim_freq` lets tied candidates displace residents that are no more popular, which adds churn and slightly lowers hit-rate instead of raising it.
- per-op-time: Both get and put latencies rose, most visibly at the tail. A one-operator change should not cost this much, so the likely cause is more evictions and replacements on the put path or measurement noise. I did not confirm which from the source diff.

## Takeaway

Tie-breaking in the admission filter is not a free lever on this trace. Estimated frequencies are low and often equal, so admitting on ties turns the frequency filter into near-unconditional admission and discards the protection that strict `>` gave established entries. The strict comparison is a better default, and any gain would have to come from a signal that separates ties, such as recency or aging, not from loosening the comparison.
