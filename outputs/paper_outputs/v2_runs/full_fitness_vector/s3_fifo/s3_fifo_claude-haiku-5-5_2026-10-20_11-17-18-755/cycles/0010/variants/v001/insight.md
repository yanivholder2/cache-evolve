# Variant v001 Insight

## Helped

- per-op-time: Put tail latency improved relative to the parent; the batched ghost prune (one pass every K pushes instead of a drain on each insert) removes per-insert expiry work from the put path, which is where the tail came from.

## Hurt

- metadata-size: The container footprint grew over the parent because the ghost cap and push counter add bookkeeping state to `S3Fifo`; the `Entry` size is unchanged, so the summary figure does not show this cost.

## Takeaway

Capping the ghost at half the main capacity with batched expiry left hit-rate exactly where the parent had it, so the gain that cycle 0009's per-insert cap showed did not carry over once its drain was batched. What this variant does teach is that batching the expiry is enough to recover put tail latency without touching the resident path, while the cap itself has no measurable hit-rate effect at this trace and size. Note: I could not inspect the overlay source in this session (no shell or directory listing was available), so this insight rests on the hypothesis text and the two fitness vectors rather than the code diff.
