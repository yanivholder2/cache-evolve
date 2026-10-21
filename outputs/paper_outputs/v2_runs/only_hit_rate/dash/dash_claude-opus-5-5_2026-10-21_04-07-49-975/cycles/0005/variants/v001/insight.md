# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Making the stash probation region two slots wide (`PROBATION_SLOTS = 2`, fresh BIP keys inserted at index 1 and FIFO-shifted to index 0) gives a fresh key one more insert to survive, but it takes that slot from the protected ex-resident region of every stash bucket. The extra hits from fresh keys with moderate reuse distance did not cover the hits lost when demoted ex-residents, which return to the normal tier after a single hit thanks to their `PROMOTE_THRESHOLD - 1` pre-credit, were evicted sooner. The regression is marginal: hit-rate was essentially flat against v003.

## Takeaway

Making probation deeper is close to zero-sum on this workload. Each slot added to probation is a slot removed from the victim cache, and on this trace a slot holding a pre-credited ex-resident is worth at least as much as one more chance for a fresh key. So v003's gain came from separating probation from the victim cache, not from how big probation is. Further work on fresh-key survival should probably avoid taking capacity from the protected region, for example by tuning `BIP_PERIOD` instead. Two caveats: this was measured on a single LIRS trace at a single cache size, and this insight relies on the hypothesis's description of the change because the overlay source files could not be listed and read in this session.
