# Variant v003 Insight

## Helped

- None. The only measured dimension, hit-rate, did not improve over the parent.

## Hurt

- hit-rate: Admitting a ghost hit straight into main only when the key had freq > 0 at small-eviction sends many ghost-hit keys back through small. Small is only capacity/20, so those keys are probably evicted again before a second access, and the reuse that the ghost entry had already proven is lost. The drop from the parent is large.

## Takeaway

A ghost hit is already strong evidence of reuse on this LIRS trace, and the freq > 0 filter throws away much of that evidence. The ghost-hit fast path into main is therefore worth keeping permissive. This is especially true with a small queue as short as capacity/20, which can't give a re-demoted key a second chance. Tightening admission was a bad direction here, so future work on ghost handling should loosen the rule or leave it as it is.
