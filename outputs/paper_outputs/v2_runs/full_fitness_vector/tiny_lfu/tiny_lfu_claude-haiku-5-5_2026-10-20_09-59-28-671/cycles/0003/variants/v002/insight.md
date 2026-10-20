# Variant v002 Insight

## Helped

- hit-rate: Hit rate moved back up relative to the parent, which is the direction H02 predicted if the precomputed-hash helpers now produce the same counter indices as the public sketch methods, so admission decisions match the original path again. The source diff could not be inspected in this session, so the link between the index change and the recovery is inferred, not confirmed.

## Hurt

- per-op-time: Both get and put became markedly slower than the parent, giving back most of v001's constant-factor gain even though H02 said to keep that gain. The likely cause is the index derivation change: H02 asks each row index to be derived from the base hash with the public path's per-row seed scheme, which adds per-row work on every sketch access that v001's shared-index helper avoided. This attribution is inferred from the hypothesis text and has not been checked against the overlay code.
- metadata-size: No change expected or observed; the sketch width and counter layout were left alone, as H02 required.

## Takeaway

Matching the public sketch's per-row seeded indexing appears to buy back the admission fidelity that v001 lost, but at the cost of the speedup that the hoisted-hash design was meant to deliver. The variant is a trade-off point, not a combined win: the next step is to check the overlay source to confirm the index equivalence and see whether the seeded derivation can be made cheaper while keeping identical indices.
