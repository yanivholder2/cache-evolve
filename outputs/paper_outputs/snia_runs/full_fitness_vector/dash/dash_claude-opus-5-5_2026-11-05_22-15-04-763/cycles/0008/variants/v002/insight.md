# Variant v002 Insight

## Helped

- per-op-time (`put` p50, p99): Both put percentiles recovered sharply, past the "about 130 ns" target in the hypothesis. This confirms the parent's diagnosis: put's cost was the full-bucket `refresh_stash_mask` re-hash on every stash removal or eviction. Replacing it with an O(1) `fps[i]` write or clear, plus a byte shift that follows the existing item shift, removes that cost from the eviction/demotion path.

## Hurt

- per-op-time (`get` p50): The median get got slightly slower, not faster than 84 ns. On the common normal-tier hit, the parent rejected the stash with one AND against a compact, likely cache-resident `stash_masks` word. This variant has to load the stash bucket's own line and run a byte loop over `fps[0..len]`. That costs more than one AND, and the lower false-positive rate does not make up for it.
- per-op-time (`get` p99): The get tail got worse instead of recovering. So `refresh_stash_mask` on promotion was not what drove the tail. The slow get path is still the stash-hit/promotion path. On that path the item shift now also shifts `fps` and runs the fingerprint check before the key compare, so per-slot exactness does not shorten it.

## Takeaway

Moving stash-presence tracking into per-slot fingerprints shows that the parent's two costs had different causes. Mask upkeep was purely a `put` problem, and O(1) slot-level maintenance fixes it completely. But the `get` fast path did better with a separate dense mask word than with a per-bucket fingerprint scan. Contrary to the hypothesis's rationale, the "extra cache line" was cheap for reads. Combining an O(1)-maintainable mask word (counting or fingerprint-derived) for the normal-hit rejection with these fingerprints for the stash-hit compare looks like the way to keep both wins. Note that `fps` grows the bucket, but the metadata-size dimension does not show this, because it measures only `Item` and `Dash`.

(Note: I could not open the overlay source in this session. This analysis rests on the hypothesis's specified changes and the parent/child fitness vectors, not a line-by-line diff. Hit rate is identical to the parent, which suggests the bit-identical lookup invariant held.)
