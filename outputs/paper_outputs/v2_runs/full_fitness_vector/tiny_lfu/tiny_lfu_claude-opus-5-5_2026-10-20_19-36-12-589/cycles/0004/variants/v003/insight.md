# Variant v003 Insight

## Helped

- hit-rate: Changing `SKETCH_WIDTH_FACTOR` from 4 to 8 gives each of the 4 independent rows twice as many columns. Each counter therefore collects about half as much collision mass from unrelated keys, so admission comparisons near the LRU head are a little more accurate. The gain is only a small fraction of what "halving overestimation" suggests. With conservative update already in place, collision noise was apparently not the main cause of wrong admissions on workload-a. Saturation at `u16::MAX` evidently didn't cost anything either.
- per-op-time: Both `get` and `put` got faster at p50 and p99, even though the sketch is the same size in bytes and does the same number of accesses (4 reads plus 4 masked writes per increment, 4 reads per estimate). The code change gives no clear mechanism for a drop this large and this consistent. The most plausible structural causes are the narrower `u16` loads and stores in the two-pass `increment_hash` (more counters per register or vector lane, which may help the compiler) and a different spread of addresses across the 2× wider rows. The parent's insight already warned that this bench drifts with the environment, though. Re-measure before crediting the `u16` switch with a speedup.

## Hurt

## Takeaway

Trading counter bits for sketch width at the same byte cost is a clean change, and metadata-size didn't move. Saturating 16-bit counters cost nothing on workload-a, so 32-bit counters were wasted space. But halving per-counter collision noise raised hit-rate only a little. Once conservative update is in place, CountMin overestimation is close to exhausted as a lever, so the remaining gap is more likely in the admission and eviction policy (for example, the plain-LRU victim choice or the lack of aging) than in sketch accuracy.
