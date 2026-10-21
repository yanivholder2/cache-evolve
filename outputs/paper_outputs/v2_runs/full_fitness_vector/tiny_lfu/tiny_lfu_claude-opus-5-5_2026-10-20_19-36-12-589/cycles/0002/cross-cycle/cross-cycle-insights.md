## Recurring wins

- Pure efficiency and layout changes that add no hot-path work are the only reliable wins so far. Cheap hashing (cycle 1) and compact `u32` link indices (cycle 2) both improved cost and left behaviour unchanged.
- The lineage moves forward by layering efficiency gains: baseline, then fast hash, then compact links. Each base keeps the hit rate of the one before it.

## Recurring failures

- Hit rate has not moved in any cycle. Every attempt to raise it fell flat or regressed: window+SLRU structure, sketch aging, and frequency-aware victim sampling.
- Adding structure or bookkeeping to the per-op path costs latency right away, and it has never paid back in hit rate. Examples are region tags, per-hit relinking, per-op aging branches, and extra hashing on misses.
- Changing the frequency signal on its own has hurt. Aging with coarser counters made more admission ties, and the strict tie-goes-to-victim rule then rejected more good newcomers.
- LRU-end victim sampling is ineffective. Estimates near the tail are nearly equal, so sampling there adds cost and gives no selectivity.

## Open questions

- Is the hit-rate ceiling coming from the admission rule (strict `>` comparison, ties going to the victim, no doorkeeper or window) rather than from the sketch or the list structure? Test changes to admission and tie handling, possibly together with aging.
- Is workload-a already near its hit-rate ceiling for a TinyLFU-family design? If so, efficiency is the only axis left to improve.
- Is the latency regression on the compact-links base real or benchmark noise? Re-measure it side by side with the fast-hash base on the same machine.
- Is the unkeyed fast hasher safe under adversarial or collision-heavy key sets?
- Would a larger admission window help, once admission or freshness is fixed?

## Per-policy notes

- TinyLFU (sketch + LRU, the base lineage): hashing was the dominant per-op cost. The put tail is most sensitive because it runs the admission contest.
- W-TinyLFU (window + SLRU): tried once at about a 1% window. It cost memory and latency and gave almost no hit-rate gain while the sketch never ages. Not worth revisiting until the frequency/admission signal is shown to matter.
- Sketch aging (Caffeine-style reset, 4-bit cap): first blocked by an infra failure, then hurt hit rate when evaluated alone. Only worth retrying alongside admission or tie-rule changes, with the reset bookkeeping kept off the hot path.
- Sampled-victim eviction: no hit-rate effect and extra per-miss cost. Dropped.
