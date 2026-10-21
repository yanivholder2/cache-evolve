# Variant v002 Insight

## Helped

- hit-rate: Replacing ClassicLRU's move-to-tail with a per-bucket CLOCK ring did more than hold LRU's hit rate. It beat it, and by more than the hypothesis's "about 1 point" tolerance. New items go into the ring with the reference bit cleared, so a key touched only once can be evicted the next time the hand passes it. Under LRU, that key would have had to age through the whole bucket from the MRU end. This acts as a cheap second-chance filter: keys that are reused stay resident longer, and one-off keys do not.

## Hurt

(No measured dimension regressed. `per-op-time` and `metadata-size` are missing from this variant's fitness vector, so the hypothesis's main claim was not evaluated. That claim is that removing `Vec` shifts on hits lowers get latency. The design should leave `Item` size unchanged, because it reuses `Item::lfu_counter` as the reference bit, but that was not measured either.)

## Takeaway

In short buckets, CLOCK's in-place reference bit replaces LRU's exact recency order and does not lose hit rate on workload-a. Inserting new items with the bit cleared appears to help, because it filters out keys that are used once. The change was meant to cut latency, so the next step is to rerun `per-op-time` on this variant. Until that runs, nothing is known about the trade this hypothesis cared about: cheaper hits, paid for with a bit sweep on each miss.
