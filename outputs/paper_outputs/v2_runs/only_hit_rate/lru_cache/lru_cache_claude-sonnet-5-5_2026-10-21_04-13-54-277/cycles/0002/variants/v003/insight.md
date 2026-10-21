# Variant v003 Insight

## Helped

- hit-rate: Replacing move-to-front with a referenced bit plus a sweeping hand lets re-referenced entries survive one hand pass, while one-shot scan entries (inserted with `referenced = false`) are evicted first. This gave a small gain over strict LRU on the LIRS workload-a trace.

## Hurt

- metadata-size: The fitness vector has no entry for this dimension, so it was not measured. The new `Node` drops prev/next links and adds a bool, so it is probably smaller, but this run does not show it.
- per-op-time: This dimension is also missing from the fitness vector. The hypothesis's claim of cheaper hits (a bit set instead of list relinking) is therefore unverified. Eviction can also loop over the arena clearing bits, which may inflate put tail latency.

## Takeaway

CLOCK second-chance is a cheap approximation of LRU that matched or slightly beat it on hit-rate for this trace. The gain was modest, so the scan-resistance claim is only weakly supported. The variant did not produce the metadata-size and per-op-time measurements, so the "less work per hit" half of the hypothesis cannot be judged. Those dimensions would need to be measured before CLOCK counts as a clear improvement.
