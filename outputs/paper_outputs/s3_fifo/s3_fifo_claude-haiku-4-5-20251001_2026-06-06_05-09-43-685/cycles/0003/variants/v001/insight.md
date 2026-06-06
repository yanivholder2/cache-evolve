# Variant v001 Insight

## Hurt

- **hit-rate**: Separating warm-ghost tracking increases memory footprint for the ghost queue while allowing re-evicted items (overflow from warm_ghost to cold ghost) to lose frequency information, reducing recovery opportunities when the working set exhibits temporal clustering.
- **metadata-size**: Adding a dedicated `warm_ghost` VecDeque field increases the S3Fifo struct size, and the capacity pre-allocation for warm_ghost (at main_size) grows overhead without delivering the hypothesized hit-rate gains.
- **per-op-time (put)**: Checking warm_ghost before cold ghost on every put, combined with overflow-management logic in evict_main, adds lookup and maintenance overhead that outweighs the benefit of occasional direct restoration from warm_ghost.

## Helped

## Takeaway

Separating ghost tracking to preserve frequency signals does not translate to hit-rate recovery. The approach increases both memory overhead and put-latency by adding a second ghost queue and associated overflow logic, while failing to improve cache hits. The root issue appears to be that warm_ghost items overflow to cold ghost faster than they get re-accessed in this workload, making frequency preservation moot; the original lazy-eviction strategy (decrement-and-reinsert) was more effective because it kept frequency information in the critical path of eviction, not isolated in a side structure.
