# Variant v002 Insight

## Helped

- None observed. hit-rate is identical to the parent's, to the reported precision.

## Hurt

- None observed in the measured dimension. The fitness vector reports only hit-rate. The extra `referenced` field and the bounded rescan loop add per-node memory and eviction-path work that nothing here measures. I could not read the overlay source in this session, so this rests on the hypothesis text.

## Takeaway

The second-chance bit only acts in `put`'s fallback path, where probation is empty and the protected head must be evicted. The unchanged hit-rate suggests that path almost never runs on this trace and size. With a 90% protected share, probation is rarely empty, so eviction nearly always comes from probation and the new flag is never consulted. A scan-resistance refinement of protected eviction cannot pay off where protected eviction is rare. Any gain would have to come from changing how probation or promotion behaves.
