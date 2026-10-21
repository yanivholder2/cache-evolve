# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Saturating count-min counters at 31 (no decay) left workload-a hit-rate exactly where it was. That means counts above 31 carried no ordering information that changed any admission or eviction decision. It also means the "stale-high demoted key squatting in probation" mechanism behind H01 is not what limits this trace. The cap only makes two keys compare differently when both sit at or above 31 and one is a probation victim candidate. That is the case `select_victim`'s min-estimate scan avoids anyway, because a capped key is never the minimum when colder entries are in the 8-entry sample. Two things follow from this neutral result. Counter ceiling is not a hit-rate lever for this policy. More usefully, the extra range is dead weight, so the next variant can shrink counter storage to 5-bit counters (same cap of 31) and gain on metadata size without expecting a hit-rate cost. A lower cap, such as 4-bit counters, would change comparisons that this test did not cover.
