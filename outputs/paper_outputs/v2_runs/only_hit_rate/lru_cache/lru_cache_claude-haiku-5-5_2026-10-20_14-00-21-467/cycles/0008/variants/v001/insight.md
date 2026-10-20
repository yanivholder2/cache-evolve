# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Halving the `hits` counter of every entry the victim scan examines weakens the frequency signal that the v003 tail window depended on, so re-referenced entries near the LRU end lose their protection sooner than under v003's undecayed counts.

## Takeaway

Aging the victim-scan counters did not help on workload-a at size 2048; the hit-rate moved down by a negligible amount from the parent, so the decay step is at best neutral on this trace. This reading is based on the hypothesis's stated change and the fitness vectors only: the overlay source could not be opened in this session, so it is not confirmed that the code matches the described halving, and a second trace or a size sweep is needed before drawing a conclusion about decay itself.
