# Cross-Cycle Insights

## Recurring wins

- **Stash-based post-eviction recovery remains the primary lever**: Both cycles validated that capturing evicted items in a bounded buffer and providing second-chance lookup on misses outperforms more sophisticated approaches (inter-bucket probing, geometry rebalancing, ordering refinements). The mechanism itself is robust across design variations.
- **Capacity expansion beats sophistication**: Simple increases to stash size (C002 v002) decisively outperformed both LRU-ordering refinement and occupancy-aware filtering, despite modest absolute gains. This suggests raw buffer size, not algorithmic optimization within existing bounds, is the current bottleneck.
- **Straightforward mechanisms beat complex redistribution**: Indirect approaches requiring architectural workarounds (inter-bucket probing, geometry shifts) failed where simple post-eviction buffering succeeded, indicating the cache's existing design constraints leave little room for dynamic redistribution.

## Recurring failures

- **Geometry-based collision reduction didn't pay off**: v003 in C001 reduced collision probability but failed to improve hit rate, implying collision pressure is not the dominant loss mechanism or per-bucket scan costs outweigh the benefit. Geometry is a blunt tool for this workload.
- **Sophisticated ordering and filtering without capacity scaling don't compound**: LRU eviction ordering (C002 v001) and occupancy-aware insertion filtering (C002 v003) both improved secondary metrics but lost to straightforward capacity expansion, suggesting they address secondary inefficiencies that are masked by undercapacity.
- **Inter-bucket migration schemes are architecturally blocked**: v002 in C001 failed because the cache's slot_legality invariant prohibits bucket migration—attempting redistribution within the existing architecture yields no benefit.

## Open questions

- **Where is the capacity saturation point?**: v002's 2.5× capacity increase (10% → 25%) yielded only 1.2% hit-rate gain, indicating diminishing returns, but no cycle has yet probed intermediate points or identified where scaling plateaus relative to the workload's actual re-use profile.
- **Do filtering and ordering become valuable at higher capacities?**: Both were bypassed by simple capacity scaling in C002, but might they compound with a larger stash (e.g., 25%+ with LRU or selective insertion)?
- **Could a hybrid combining stash and geometry improvements outperform either alone?**: C001 left this unresolved; C002 did not test geometry + capacity.
- **What is the efficiency frontier between capacity, ordering, and filtering?**: Is the sweet spot capacity-dominant, or does a balanced combination perform better across different workload re-use profiles?

## Per-policy notes

- **Stash (buffer overflow + second-chance lookup)**: Proven winner across all variants tested. Capacity scaling remains productive; ordering and filtering are secondary levers that have not yet compounded effectively with capacity growth.
- **LRU eviction ordering within stash**: Tested in C002 v001, did not outperform baseline FIFO at 10% capacity. Effectiveness may depend on capacity scale; not yet tested at higher capacities.
- **Occupancy-aware selective insertion**: C002 v003 reduced churn and improved some metrics but lost to straightforward capacity expansion. Synergy with larger buffers remains unexplored.
- **Geometry rebalancing (collision reduction)**: C001 v003 showed no benefit despite theoretical merit, suggesting collision is not the binding constraint and per-bucket scan costs dominate. Deprioritized in both cycles; unlikely to be productive without evidence of collision pressure.
