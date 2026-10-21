# Variant v002 Insight

## Helped

- hit-rate: Replacing strict LRU with CLOCK did not cost hit rate on workload-a at size 2048; it came out slightly ahead of the parent. This suggests the trace benefits from second-chance's coarser recency, where any entry referenced since the last sweep is protected equally, instead of LRU's exact ordering.

## Hurt

(none measured)

## Takeaway

On workload-a, dropping the `prev`/`next` links for a one-bit `referenced` flag and a clock hand kept hit rate at LRU's level, and it was slightly better. So the exact LRU ordering was not earning its keep. This settles the hypothesis's main risk. However, v002's fitness vector has only the `hit-rate` dimension; `metadata-size` and `per-op-time` are missing. The hypothesis's real goals were the smaller node, which a compile-time `size_of::<Node<u64,u64>>() <= 24` assert enforces, and cheaper hits, since `get` now does one flag store instead of list splices. Neither is confirmed by measurement, so re-run the eval for those two dimensions before treating v002 as a win on them. Also watch `put` p99: when many entries are referenced, `find_victim` can sweep up to the whole ring.
