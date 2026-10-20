# Variant v002 Insight

(Note: overlay source could not be listed in this session; this is based on the hypothesis and fitness vectors.)

## Helped

- hit-rate: The hypothesis promised identical bucket contents, yet hit rate rose. So the fused scan/free-slot/swap-insert did not exactly match the parent's insertion or eviction behavior. Most likely the "first empty slot, else overwrite last victim, then swap to len/4" rule changed which entries survive.

## Hurt

- per-op-time: Put p50 and p99 both got slower, which is the opposite of the goal of the fusion. Get p50 and p99 also got slower even though the get path was meant to be untouched. This points to code-layout or inlining effects, or to the fused loop doing extra work per slot, such as tracking the empty index on every iteration.

## Takeaway

Merging the existence scan and the free-slot search into one loop did not lower put latency. It slowed both put and get, and it silently changed eviction behavior, so the "identical contents" invariant did not hold. Micro-fusing the put path is not a safe latency win here. The hit-rate gain suggests that where a new item is placed in the bucket matters more than how fast the scan runs.
