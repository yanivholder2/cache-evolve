# Variant v002 Insight

## Helped

- hit-rate: Replacing the warm-pool fallback from LRU position 0 to minimum-`lfu_counter` selection means that when a 16-slot bucket is fully warm, the evicted item is the one accessed least often rather than the merely oldest — preserving higher-value entries that happen to sit at the front of the recency queue.

## Hurt

## Takeaway

The cold-first stage was already proven on wide-bucket geometry; this variant isolates the warm-pool fallback on baseline narrow (128 × 16) geometry where cold pool exhaustion is routine. The substantial hit-rate improvement confirms that the LFU fallback — not just cold-first eviction — is independently valuable: once all 16 slots have been touched at least once, a frequency signal is far more discriminating than LRU position for deciding which warm item to drop.
