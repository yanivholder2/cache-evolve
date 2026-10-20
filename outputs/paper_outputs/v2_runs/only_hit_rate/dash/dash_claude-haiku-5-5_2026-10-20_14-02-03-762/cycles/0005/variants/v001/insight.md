# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The hypothesized ghost set readmits a recently evicted key on its first return and skips the seen-in-stash wait, so returning keys displace current residents sooner than the parent's gate would allow, and on this trace that trade-off came out slightly negative. The mechanism is inferred from the hypothesis text; the overlay diff could not be inspected in this pass, so it is not confirmed in the shipped code.

## Takeaway

Giving evicted keys a fast path back in through a ghost history did not raise hit-rate on LIRS workload-a at size 2048 and moved it slightly down from the parent, which suggests early readmission competes with residents that were still earning their place. This describes the intended design only: the overlay was not readable here, and the fitness vector has no per-op-time or metadata-size entries, so the ghost set's cost is unmeasured. It is one trace at one capacity.
