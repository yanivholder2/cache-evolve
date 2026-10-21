# Variant v001 Insight

## Helped

- hit-rate: Inserts into a full cache are now dropped when the candidate's sketch estimate is below the probation victim's. This keeps some one-touch scan keys from displacing probation entries that have been seen more often, which gives a small gain over the parent.

## Hurt

- hit-rate: Misses are also counted in the sketch, so a rejected key still builds frequency and is admitted on a later retry. The gate can also reject a genuinely new hot key whose count is still low. Both effects limit the gain.

## Takeaway

TinyLFU-style admission added on top of a segmented LRU with a 95% protected share gave only a marginal hit-rate improvement. Segmentation had probably already filtered most scan pollution, since one-hit keys only ever enter the small probation segment. The extra sketch work (hashing, 4 counters per access, periodic halving) buys little, so further gains likely need something other than a stricter admission gate.
