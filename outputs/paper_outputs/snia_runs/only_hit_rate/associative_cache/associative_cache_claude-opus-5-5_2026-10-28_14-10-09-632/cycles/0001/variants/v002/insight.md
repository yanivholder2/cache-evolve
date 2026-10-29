# Variant v002 Insight

## Helped

(none — on web_0 at the larger size hit-rate stayed about level, which is noise, not a gain)

## Hurt

- hit-rate: Inserts still land at the MRU tail through the unchanged `Bucket::put`, but a hit now only moves an item up `len/8` slots. So every brand-new key outranks a key that was just re-referenced, which inverts the frequency gradient the hypothesis wanted. A reused item that has aged near index 0 gets only a short lift and is still evicted soon. The loss is largest on hm_0 at the larger cache size, where the baseline's full jump to MRU was holding on to a working set with moderate reuse distance.

## Takeaway

Limiting how far a hit can promote an item, while new items are still inserted at the very top, punishes reuse instead of rewarding it. On both MSR traces, bounded promotion did worse than the classic jump to MRU. The hit-rate drop was flat-to-small on web_0 and clear on hm_0. A graded-promotion scheme only makes sense if new items are also inserted lower in the bucket (for example at or below the promotion step), so that a hit is worth more than an insert. Per-op-time was not measured for this variant, so the side benefit of shorter memmoves claimed in the hypothesis is still untested.
