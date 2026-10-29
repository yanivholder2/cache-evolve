# Variant v002 Insight

## Helped

- hit-rate: Hits now bump `lfu_counter` by one each time, up to 3, instead of setting a single bit. Each sweep pass only takes one off. So a key hit several times survives several passes of the hand, while a one-touch key (inserted at 0) still goes first. Hit rate rose a little at all four trace/size points, and rose most at the larger 131072 size. That fits the idea that the extra frequency levels matter most when a bigger skewed hot set is competing with scan traffic.
- per-op-time (put p99): The new `clock_sweep` stops after `SWEEP_LIMIT = 16` slots. In the parent, the sweep had no limit and could clear bits across up to two full passes of a 128- or 1024-wide bucket. That worst-case walk was what drove the parent's put tail, and the limit removes it. This is the clearest win of the change.

## Hurt

- per-op-time (put p50): A typical eviction now does more work. With 2-bit counters, a hot slot needs up to three passes to reach 0, so the window has fewer zeros and the sweep finds one less often. More sweeps run the full 16 slots, and each slot gets a decrement-and-write plus a compare for the running minimum. In the parent, clear-on-insert meant a zero bit usually turned up after a few slots.
- per-op-time (get p50 / summary): The only change on the get path is a load, compare and conditional increment, where the parent did a blind `lfu_counter = 1` store. That is a very small change, so the size of the get p50 regression is surprising. It may be noise in the benchmark, or a side effect of hot keys now holding their slots for longer, which changes where they sit for the linear `get_position` scan. The eval artifacts do not show which. Because the summary is get p50, the summary got worse even though the tail improved.

## Takeaway

A hard limit on the CLOCK sweep fixes the put tail without giving up hit rate. Adding counter levels to GCLOCK buys only a little extra hit rate here. That gain has a cost: the sweep finds fewer zero-counter slots, so the typical put gets slower. Hit-rate gains from reworking the replacement rule are now very small. A follow-up after speed should keep the 16-slot limit and the clear-on-insert, and go after the linear `get_position` key scan, which drives both get p50 and the summary dimension.
