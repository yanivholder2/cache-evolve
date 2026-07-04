# Variant v002 Insight

## Helped

_(no measured dimension improved)_

## Hurt

_(no measured dimension worsened)_

## Takeaway

Dropping `timestamp: Instant` from `Item<K, V>` had no effect on `hit-rate`, which
was expected — the active workload runs ClassicLRU, a policy that never reads the
timestamp field. The hypothesis's primary claim was a `metadata-size` reduction
(40 → 24 bytes per item), but that dimension was not captured in v002's evaluation,
so the structural benefit of the removal remains unconfirmed by measurement. Future
cycles should ensure the `metadata-size` and `per-op-time` evaluators run against
this variant before treating the size saving as established.
