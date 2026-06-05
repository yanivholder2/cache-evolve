# Variant v002 Insight

## Helped

- hit-rate: Replacing the conditional ghost-set path with an unconditional second-chance `promote_to_target(Front)` on every stash eviction means semi-warm items gain a slot in the target tier immediately, rather than only if the same key happens to be re-inserted while the ghost window is still open.

## Hurt

## Takeaway

The ghost set's weakness — confirmed here — is that it is event-driven: it only fires when a previously evicted key is explicitly re-inserted by the workload. For workload-a's stable hot set, that re-insertion event may rarely or never arrive for the keys that cycle through stash overflow. The second-chance mechanism removes the external trigger entirely: every stash-evicted item gets one free cycle of life at the LRU end of its target bucket, letting reuse probability rather than workload timing determine survival. The measurably larger hit-rate gain over the ghost set's near-noise movement shows that the bottleneck was the conditional firing, not the insertion position.
