# Variant v003 Insight

## Helped

- **hit-rate**: Increasing MAX_FREQ from 3 to 5 extends the eviction protection window in the main queue, allowing moderately-reused items more frequency-decrements before permanent removal—the extra cycles of second-chance protection recover items that contribute measurable reuse but would be discarded too early under the tighter frequency ceiling.

## Hurt

(none)

## Takeaway

Deeper frequency protection compounds with tight small-queue admission. The 1/8 admission gate filters through only high-confidence repeaters; raising MAX_FREQ from 3 to 5 gives these survivors additional eviction resilience. The modest but consistent improvement shows that the workload contains items with moderate reuse patterns (3–5 accesses) that justify the extended main-queue protection window. This variant confirms that within the S3-FIFO architecture, frequency depth and admission selectivity are complementary levers—tightening one without strengthening the other would leave gains on the table.
