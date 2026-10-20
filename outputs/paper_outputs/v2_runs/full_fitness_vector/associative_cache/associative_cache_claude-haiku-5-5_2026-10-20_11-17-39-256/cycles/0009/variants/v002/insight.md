# Variant v002 Insight

## Helped

(none: no dimension improved)

## Hurt

- per-op-time: Halving moved from one global pass to lazy per-bucket catch-up, so the tail cost now lands inside ordinary `put` calls on full buckets. The put tail got worse rather than better, which means the global pass was not the main source of the put tail.
- per-op-time: The hypothesis puts the lazy catch-up on the put-side scan, but it also lets `get` trigger catch-up on any bucket whose epoch is behind. That extra check and halving on the read path is the most likely reason `get` regressed too, since `get` was not meant to be affected.

## Takeaway

Replacing the periodic global halving with lazy per-bucket halving kept hit-rate and metadata size unchanged, which confirms the decay schedule was preserved, but it made both `get` and `put` slower in the tail. The lesson is that the whole-structure halving pass was not what drove the put tail; the cost of decay is paid per operation wherever it runs. I could not inspect the overlay source from this session, so the attribution of the `get` regression to the read-path catch-up is inferred from the hypothesis text and not confirmed in code.
