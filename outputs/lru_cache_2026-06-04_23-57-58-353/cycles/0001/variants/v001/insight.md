# Variant v001 Insight

## Helped

- hit-rate: Splitting the list into a probationary segment (new inserts) and a protected segment (entries accessed at least twice) keeps one-hit scan traffic from displacing repeatedly-accessed keys, so popular items survive much longer before eviction.

## Hurt

_(none — no dimension regressed)_

## Takeaway

The SLRU change confirms that the baseline LRU was meaningfully hurt by scan pollution on workload-a: routing all new inserts through a small probationary buffer and only promoting entries that receive a second access raised hit rate substantially. The 80 % protected / 20 % probationary split gave the hot working set enough room to absorb cold-traffic bursts without being evicted.
