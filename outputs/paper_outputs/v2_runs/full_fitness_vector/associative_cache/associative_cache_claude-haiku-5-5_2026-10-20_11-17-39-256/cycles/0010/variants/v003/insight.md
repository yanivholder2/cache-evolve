# Variant v003 Insight

## Helped

- hit-rate: Lowering the saturation ceiling to 3 did not cost hit-rate; the small gain over the parent is consistent with the hypothesis's expectation that the victim search still has enough frequency range to separate re-referenced keys from one-shot inserts at this ceiling.

## Hurt

- per-op-time: The get path got markedly slower at both median and tail, which is the opposite of the hypothesis's goal of cutting get-side write traffic; the skipped store is not paying for whatever extra work the lower ceiling now causes on hot hits.
- per-op-time: The put path also got markedly slower at both median and tail even though the hypothesis says the put path is unchanged apart from which slot holds the minimum, so the put regression points to the lower ceiling changing victim choice or the frequency distribution in a way that adds cost, not to a store-skip effect.

## Takeaway

Compressing the saturation ceiling to 3 keeps hit-rate level on this trace, so the frequency range that the victim search relies on is not the binding constraint at 2048 entries. The timing result runs against the hypothesis: the get and put regressions suggest the store skip is not the dominant cost, and that a lower ceiling changes access patterns enough to hurt latency. This read is based on the hypothesis text and the fitness vectors; the overlay source was not inspected, so the mechanism behind the timing regression is unconfirmed.
