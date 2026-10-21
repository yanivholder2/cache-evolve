# Variant v001 Insight

## Helped

- metadata-size: Not measured in this variant's fitness vector, so no benefit is confirmed. By construction, replacing the `Option<usize>` links with `u32` plus a `NIL` sentinel should cut `Node<u64,u64>` from 48 to 24 bytes, and `HashMap<K, u32>` should shrink the map entries.

## Hurt

- metadata-size, per-op-time: The vector reports only hit-rate, and the parent reports all three dimensions. The change itself shouldn't hurt these, so the likely cause is an evaluation or harness gap. Either way, the claimed gain can't be verified from this variant's output.

## Takeaway

The `u32`-index and sentinel rewrite kept LRU semantics intact: hit-rate is unchanged from the parent, and the `unlink`/`push_back` logic is a faithful translation. The hypothesis's actual target, node size, has no recorded result, so the metadata-size win stays a prediction until the evaluator emits those dimensions. The next step is to fix the evaluation so it reports them.
