# Variant v002 Insight

## Helped

## Hurt

- per-op-time: The `put` tail got worse, and the `put` median did not improve, despite H02 predicting a drop from folding the existing-key check and insert into one `HashMap::entry` call. On a miss, the restructured path still pays for a victim removal at capacity, so the saved second key hash is partly offset by the extra `select_victim` and remove work that the entry borrow forces into a separate step.
- per-op-time: The `get` tail rose even though H02 leaves `get` untouched, so this movement is not explained by the `put` change itself and is more likely layout or measurement noise; it should not be credited to or blamed on the entry-API rewrite.

## Takeaway

Collapsing the `put` miss path onto a single key-map lookup did not reduce `put` cost on workload-a, and the hit-rate staying fixed confirms the eviction order was preserved. The result suggests the second hash on a miss was not the dominant cost in `put`, and that the forced pre-select-then-remove ordering around the vacant entry adds back enough work to cancel it. Note that the source diff could not be inspected in this run, so the reasoning above rests on the hypothesis text and the fitness vectors.
