# Variant v003 Insight

## Helped

- hit-rate: Replacing plain LRU with a two-segment SLRU structure plus a ghost admission mechanism captured both in-cache frequency (the 80/20 protected/probationary split) and cross-residency warmth (the ghost set routing evicted-then-re-arrived keys directly into protected), yielding a large gain on a LIRS trace that exercises exactly this kind of recency-vs-frequency tension.

## Hurt

## Takeaway

The ghost set's value on this workload is that evicted probationary keys often reappear before the ghost window expires, and routing them straight into protected on re-arrival prevents them from being filtered out a second time by the one-hit admission rule. Because the parent is a plain LRU, this variant bundles two changes — SLRU segmentation and ghost admission — so the measured gain reflects both; future variants should isolate SLRU-only against this variant to quantify how much of the improvement the ghost set contributes on its own.
