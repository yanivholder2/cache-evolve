# Variant v002 Insight

## Helped

- hit-rate: Replacing move-to-MRU on hit with a per-node `referenced` bit (second-chance eviction) keeps recently hit entries alive through one eviction pass, which the hypothesis expected to be roughly neutral and instead moved the hit rate up on workload-a at size 2048.
- per-op-time (put): The overwrite path now sets `referenced = true` instead of unlinking and re-pushing the node, which removes list surgery from the common put-existing case and lowers put p50.

## Hurt

- metadata-size: Adding the `referenced: bool` field to `Node<K, V>` grows each node, and with 8-byte-aligned `u64` keys and values the extra byte pads the node up to the next alignment step.
- per-op-time (get): The hit path no longer unlinks and re-pushes the node, yet get p50 rose, so the removed list work did not translate into a faster get; the change alone does not explain this and the result may be measurement noise or an unrelated cost on the get path.
- per-op-time (put): The eviction path now scans from `head` past referenced nodes, which adds variable work on a miss and is the likely source of the worse put p99.

## Takeaway

Swapping exact move-to-MRU for a referenced bit gave a hit-rate gain and a cheaper put p50, but the hypothesis's main claim, a faster get, did not hold, and the second-chance scan on eviction traded that for a worse put tail. The source diff could not be inspected in this session, so the attribution of each bullet to the code change is inferred from the hypothesis text and should be checked against the overlay before the result is treated as settled.
