# Variant v001 Insight

## Helped

- hit-rate: replacing the hardcoded front-of-vector eviction with a full-bucket LFU scan (lfu_counter incremented on every hit) protected high-frequency Zipfian hot items from being evicted during temporary positional displacement by cold-item puts, while the position-order tie-break preserved LRU behaviour among equally-cold items.

## Hurt

*(no dimensions hurt)*

## Takeaway

Layering access-frequency tracking onto the existing move-to-back discipline gives ClassicLRU meaningful protection against the characteristic failure mode of pure recency eviction on skewed workloads: a short burst of cold inserts can age a frequently-used item to the eviction front, permanently evicting it. The `lfu_counter` field was already carried on every `Item` at zero cost, so this change activated latent capacity in the data model with no structural overhead.
