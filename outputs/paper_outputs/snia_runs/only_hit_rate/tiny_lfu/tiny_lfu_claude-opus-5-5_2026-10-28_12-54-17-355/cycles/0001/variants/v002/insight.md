# Variant v002 Insight

## Helped

- hit-rate: Splitting the single LRU into a 20% probation / 80% protected SLRU and always taking the admission-contest victim from the probation LRU means keys that have been re-referenced since admission no longer drift to the eviction point. The sketch therefore compares newcomers only against not-yet-proven residents. Every trace×size cell improved. The gain was largest on web_0 at both sizes and smallest on hm_0 at the large size, where the baseline's single LRU already kept most of the reused working set.

## Hurt

(none measured; v002's fitness vector reports only hit-rate. `Node` gains a `Region` tag, which likely adds a word of padding to `Node<u64,u64>`, and `on_hit` can do a second unlink/relink when demoting from protected. metadata-size and per-op-time should be re-measured before this is treated as a free win.)

## Takeaway

When TinyLFU picks its victim, which resident it compares against matters about as much as the frequency sketch. Taking that victim from the probation segment, so keys already re-referenced are shielded, gives a consistent, low-risk hit-rate gain on top of the unchanged CountMinSketch and admission rule. The gain is modest and depends on the workload: it is strongest where web_0's scan-like one-hit traffic used to push reused keys out of a plain LRU, and it shrinks once the cache is large enough to hold hm_0's reuse set anyway.
