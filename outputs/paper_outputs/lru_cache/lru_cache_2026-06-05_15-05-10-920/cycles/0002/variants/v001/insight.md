# Variant v001 Insight

## Hurt

- hit-rate: Reversing tie-breaking direction from LRU to MRU (head-to-tail → tail-to-head scan) harms the workload; among low-frequency items, evicting the oldest is better than evicting the most recent, suggesting older arrivals lack temporal reuse potential while newer arrivals have higher re-access probability.

## Takeaway

The hypothesis correctly identified that tie-breaking in frequency-based eviction matters, but misidentified the correct direction. For workload-a, when multiple items share the minimum frequency, the old strategy of scanning from head (LRU end) and evicting the first match outperforms the new strategy of preferring the tail-most (MRU end) victim. This shows that on this workload, temporal locality within the low-frequency tier actually favors age: older arrivals are genuinely "colder" and less likely to be re-accessed than recently-added items, making pure LRU-ordered tie-breaking superior to recency-biased selection.
