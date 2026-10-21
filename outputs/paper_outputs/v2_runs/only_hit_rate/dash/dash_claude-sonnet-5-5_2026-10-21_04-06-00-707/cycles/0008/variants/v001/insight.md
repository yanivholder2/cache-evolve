# Variant v001 Insight

## Helped

- hit-rate: The hypothesis adds a direct-mapped ghost table that remembers an evicted victim's counter. Keys that return after eviction are readmitted with a counter of at least 1 instead of 0, so they are less likely to be evicted again right away. Hit-rate on workload-a rose over the parent, but only by a very small margin.

## Hurt

- None measured. hit-rate is the only reported dimension. The ghost table adds memory and a write on every eviction. It also adds a lookup on every admitted miss. No dimension captures those costs. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Restoring history for recently evicted keys gave a gain that is close to noise. That suggests the two-bit counter and the second-sighting admission filter already catch most of the returning keys on this trace. The ghost table's extra state buys little on workload-a at this cache size. The result comes from one trace and one size, so it does not show whether the idea would help on workloads with longer reuse distances.
