# Variant v001 Insight

## Helped

- hit-rate: No movement. The head-adjacent early return only changes eviction order for the second-from-head node, and on this trace that changed nothing measurable. It is neutral, not a gain.
- metadata-size: No movement. The change adds a branch and no fields, so node and cache layout are the same as the parent's.

## Hurt

- per-op-time: Both get and put are slower than the parent. The added check (is the node in the protected segment, and is it the head or the head's `prev`) costs a compare on every hit. Few hits land that close to the head, so the check rarely saves a relink. Put also got slower even though it was not the target. That suggests the extra branch sits in a shared touch path, or that the code layout shifted, or that the measurement is noisy.

## Takeaway

Skipping relinks for head-adjacent protected hits did not pay off. The skip applies only to the head and the one node behind it, which is a tiny fraction of hits under a skewed or looping workload. The check's overhead exceeds the relinks it saves, and hit-rate is unchanged. A cheaper promotion path would have to avoid relinks for a much larger share of hits, or avoid adding work to the hot path. I could not read the source files in this session, so this reading rests on the hypothesis and the fitness vectors alone.
