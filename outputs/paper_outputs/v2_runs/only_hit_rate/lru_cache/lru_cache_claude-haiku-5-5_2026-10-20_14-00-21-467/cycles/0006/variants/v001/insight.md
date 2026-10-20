# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Placing new admissions at the LRU tail (with only a 1-in-32 MRU insert) means one-touch-then-reused keys start as eviction candidates, so the re-reference that would have protected them can arrive after they are already gone. This is the only dimension measured, and it fell relative to the parent.

## Takeaway

Stacking tail insertion onto the parent's lookup collapse did not reproduce the hit-rate gain the hypothesis predicted; hit-rate on workload-a moved down, so the insertion-position change is the likely cause, since the hypothesis left the hit path and eviction victim selection unchanged. The overlay source could not be inspected in this run, so this reading rests on the hypothesis text and the fitness direction, not on the code diff. The per-op time effect the rationale expected to measure is absent from the fitness vector, so this variant says nothing about the hit-path speed-up.
