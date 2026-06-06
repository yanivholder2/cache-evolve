# Variant v002 Insight

## Hurt

- hit-rate: Periodic decay aggressively discards frequency signal, disadvantaging proven hot items relative to new arrivals and degrading admission decisions.

## Takeaway

Periodic sketch decay intended to make the cache responsive to recent patterns, but right-shifting all counters every capacity puts removes signal too quickly for this workload. The baseline's monotonic accumulation, despite risking saturation, preserves crucial frequency information that matters more than resetting to favor novelty. Decay works against the TinyLFU design: items that earned high frequency counts genuinely deserve protection, and halving them periodically gives newcomers unearned influence before they've proven themselves hot.
