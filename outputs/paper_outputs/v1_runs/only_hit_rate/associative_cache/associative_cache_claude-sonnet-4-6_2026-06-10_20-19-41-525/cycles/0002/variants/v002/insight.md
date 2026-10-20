# Variant v002 Insight

## Helped

- hit-rate: Preferring counter==0 (cold) items as eviction candidates shields any item that has received at least one GET hit from being displaced by one-hit scan insertions, directly cutting scan-pollution evictions on the LIRS workload-a trace.

## Hurt

## Takeaway

Adding a lightweight "has been accessed" threshold to `eviction_candidate_index` — scanning for the first cold item before falling back to the standard LRU front — produced a large hit-rate gain on top of v001's geometry change. The wider 64-slot buckets from v001 were a necessary prerequisite: they keep enough cold items resident per bucket to absorb scan traffic without immediately exhausting the cold pool and collapsing back to plain LRU. Together, the two changes implement a minimal second-chance policy that separates scan-once items from recurrently accessed ones without any ghost cache or separate queue.
