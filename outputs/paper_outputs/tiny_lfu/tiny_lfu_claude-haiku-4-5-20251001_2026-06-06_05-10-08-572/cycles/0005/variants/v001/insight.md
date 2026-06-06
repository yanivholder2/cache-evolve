# Variant v001 Insight

## Helped

- **hit-rate**: Frequency-aware victim selection in a 5-item candidate pool improved hit-rate by targeting truly cold items rather than purely LRU-ordered evictions, aligning decisions better with actual access frequency patterns.

## Hurt

- **per-op-time**: Scanning a window of items to find minimum frequency adds variable work during eviction, causing tail latency (p99) on put operations to degrade significantly (+96%) while median latency remains nearly flat.

## Takeaway

Adding frequency awareness to victim selection through a small candidate pool (N=5) successfully improves hit-rate by ~0.3%, confirming that breaking strict LRU order in favor of frequency signals is beneficial on depth-3 sketches. However, the scanning loop introduces latency variance—particularly at the tail—exposing the cost of trading off O(1) LRU simplicity for better eviction accuracy. This variant validates frequency-aware selection as a viable lever but reveals that the tail latency cost may outweigh hit-rate gains for latency-critical workloads.
