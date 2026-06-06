# Variant v001 Insight

## Helped

- hit-rate: Frequency-recency hybrid scoring enables retention of recently-accessed items even if their absolute frequency is lower, reducing the number of hot items unnecessarily evicted by pure LRU.

## Hurt

(none)

## Takeaway

Adding a frequency-recency hybrid victim-selection function (score = age / (frequency + 1)) delivered a measurable improvement in hit-rate. By weighing both how long ago an item was accessed and its observed frequency, the policy avoids the pure-LRU pitfall of evicting a recently-touched item just because it arrived earlier. The full-cache iteration to compute scores has a small overhead, but on this workload the improved discrimination between truly cold items and temporarily idle hot items nets a positive return.
