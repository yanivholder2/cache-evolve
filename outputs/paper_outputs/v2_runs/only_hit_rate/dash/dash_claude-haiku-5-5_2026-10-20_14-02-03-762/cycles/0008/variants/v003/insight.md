# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Shrinking the probing tier by 128 slots and giving them to the stash, with admission and promotion unchanged, left fewer probing slots for entries that were still being reused, so the stash gain did not offset the loss on workload-a.

## Takeaway

Trading probing-tier capacity for stash capacity lost hit-rate on LIRS workload-a at size 2048, so the probing tier is not oversized and stash pressure is not the binding constraint the rationale assumed. The next capacity experiment should go the other way, or leave the split alone and target the admission path.
