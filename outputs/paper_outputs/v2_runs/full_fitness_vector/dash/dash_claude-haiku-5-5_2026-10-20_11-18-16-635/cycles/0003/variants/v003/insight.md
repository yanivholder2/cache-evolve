# Variant v003 Insight

## Helped

- put: Keys rejected on their first miss never reach the table write or the eviction that would follow it, so put latency fell at both p50 and p99.

## Hurt

- hit-rate: Delaying admission of every new key by one miss means workload-a loses hits on keys that are reused soon after their first touch, which the baseline's admit-on-first-miss path captured.

## Takeaway

Gating admission behind a one-bit doorkeeper cheapens puts but costs hit rate on workload-a, so this trace depends on first-touch admission for keys with short reuse distance. The hypothesis's claim that scan-heavy traces gain was not tested here, since only workload-a at size 2048 was measured, so the premise that admission filtering helps remains open. The get-side latency drop is not explained by this change, since the hypothesis left the get path untouched, and it should be rechecked before being credited to anything.
