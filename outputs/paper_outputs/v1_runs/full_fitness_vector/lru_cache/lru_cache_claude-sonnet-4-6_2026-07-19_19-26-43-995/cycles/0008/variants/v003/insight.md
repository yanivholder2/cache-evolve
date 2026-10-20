# Variant v003 Insight

## Helped

- **hit-rate**: Reducing `prob_cap` from `capacity/4` to `capacity/16` reserves 93.75% of slots for the protected queue, sharply curbing churn of frequently-accessed entries and reproducing the hit-rate peak that cycle/0007/v001 measured from a different parent.

## Hurt

- **per-op-time**: The parent's get p50 was sub-100 ns (measuring as 0 at timer resolution), but v003's get p50 rose to 100 ns — matching the regression seen in cycle/0007/v001 — indicating that the larger protected quota triggers more frequent LRU boundary enforcement (demotion from protected to probationary whenever `prot_len > prot_cap`) and that this overhead is attributable to the quota change itself, not to any parent measurement anomaly.

## Takeaway

Starting from the clean link-array baseline of v002 confirmed that the hit-rate gain belongs to the `capacity/16` quota setting and is robust across parents, but it also ruled out the hypothesis that v002's zero-floor get latency would survive the change: the per-op-time regression travels with the quota, not with the parent. The 93.75% protected quota is a consistent hit-rate winner but carries a real, quota-induced latency cost that cannot be escaped by parent selection alone.
