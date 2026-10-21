# Variant v001 Insight

## Helped

(No measured dimension improved. The fitness vector contains only `hit-rate`, and it matched the parent exactly. The intended win is that each bucket's ghost ring shrinks from four `Option<K>` slots to four `u16` fingerprints, and the miss-path scan now compares `u16`s instead of `Option<K>`. That win is on metadata-size and per-op-time, and neither is in this fitness vector, so it is real by construction but unmeasured here.)

- hit-rate (held, not improved): no movement, which is the outcome H01 predicted. Swapping each stored key for its top-16-bit mixed fingerprint left admission decisions on workload-a at 2048 the same as v003's. At roughly 4/65535 false positives per miss, collisions are too rare to change which newcomers get counter 1, and a false positive can only raise a newcomer's counter from 0 to 1 anyway.

## Hurt

(No measured dimension regressed.)

## Takeaway

The ghost ring on this GCLOCK/LIP line only needs to answer "was this key evicted recently?", never "which key was it?". A 16-bit fingerprint, with 0 reserved to mean "empty", answers that well enough that the admission-time hit-rate benefit H03/v003 found carries over unchanged. This makes v001 the better base: same hit rate, about one-eighth of the per-bucket ghost state. It also costs only a cheap FNV-1a hash plus a mix step per miss, rather than relying on `K: Copy` storage. Whether the memory and per-op-time savings show up in practice can't be judged from this evaluation. The next eval should add metadata-size and per-op-time to the fitness vector. Otherwise, by the refutation clause, this variant can't be told apart from its parent.
