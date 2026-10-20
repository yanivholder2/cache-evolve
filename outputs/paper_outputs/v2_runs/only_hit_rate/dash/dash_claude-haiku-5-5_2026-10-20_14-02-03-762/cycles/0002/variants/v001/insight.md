# Variant v001 Insight

## Helped

## Hurt

## Takeaway

H01 made the stash-hit landing slot conditional on target-bucket occupancy: keep `InsertPosition::Back` (v002's MRU landing) when the bucket has a free slot or its LRU-end resident is stash-origin, and fall back to `Front` only when the bucket is full and its LRU-end resident is a normal-tier item. On the LIRS workload-a trace at size 2048, hit-rate did not move relative to the parent v002, so the occupancy condition showed no measurable benefit here. The most likely reading is that the full-bucket, normal-tier-LRU case that would select `Front` rarely arises on this trace, so the variant behaves like v002. This is an inference from the hypothesis and the unchanged fitness vector, not from the diff: the overlay source could not be read in this session, so I have not confirmed that the change is wired in as described.
