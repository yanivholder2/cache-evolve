# Variant v003 Insight

## Helped

- per-op-time: get and put both got faster at p50 and p99, with put's tail improving most. The likely cause is how H03 was built, not the pairing idea itself. Each pair is one physical bucket backed by one contiguous `Vec` (halves split at `len0`). The key is hashed once in `AssociativeCache` and that `h` is passed into `pair_get`/`pair_put`, which pick the home half from a hash bit. The policy's access hook works on a slice in place. So the "second probe" is a scan of the next part of the same allocation, and demotion/promotion are `Vec` shifts inside that bucket, not moves between buckets. The baseline source isn't in the parent overlay, so the exact path-length savings can't be traced line by line.

## Hurt

- hit-rate: hit rate came in slightly below the baseline instead of above it. Each pair still holds the same total of `2 * bucket_size` items, but it is now managed as two separate LRU halves. Those halves are tied together by three rules: the timestamp-gated demotion, the "evicted guest is always dropped" rule, and the promotion swap. Together they act no better than per-bucket LRU on workload-a, and possibly a little worse. One cause: a hot half can fill its partner with demoted items that push out the partner's own native items. Another: promoting an item moves home's LRU item into the partner, which displaces the partner's ordering.

## Takeaway

Sharing adjacent buckets as a victim area with promote-on-hit did not raise hit rate. At this size and on this trace, the extra flexibility within each pair is cancelled by the split-LRU bookkeeping. The speedup appears to come from the implementation: one contiguous `Vec` per pair, one hash per op, and in-slice policy updates. That speedup is worth carrying into later variants on its own, separate from the demotion/promotion policy. Note also that the variant departs from H03 as written: it does not use `h ^ 1` across `n` buckets. Instead it groups pairs into physical buckets and picks the home half from a hash bit. That changes which keys share a partner compared with the hypothesis.
