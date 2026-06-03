# TinyLFU Subject

The TinyLFU subject implements an admission filter based on a compact frequency sketch.
The baseline is an LRU resident cache that only admits a new item when the filter
estimates its frequency as higher than the least-recently-used victim.

This design follows the TinyLFU paper's admission policy and is implemented in Rust
for the cache-evolve subject framework.
