# Variant v003 Insight

## Helped

- hit-rate: The doorkeeper gate rejects a put miss on its first sighting, so one-shot keys never reach the victim scan and stop displacing resident items on workload-a.

## Hurt

- metadata-size: The bitset is held as a boxed array, so the struct gains a pointer and length while the bit storage itself sits on the heap and is not counted by the size probe.
- per-op-time: The get path regressed slightly even though the hypothesis left it untouched, which points to struct layout or extra indirection from the new field rather than to the admission logic.
- per-op-time: Put tail latency rose, which fits the periodic clear of the whole doorkeeper every N put misses, a burst of work that the gate's skipped victim scans do not offset at p99.

## Takeaway

Admission filtering on the miss path is a real lever for hit-rate on workload-a, because it targets the one-shot pollution that the bucket-level policies could not remove. Its costs are structural rather than algorithmic: the doorkeeper's storage, its clearing schedule, and the added field on the cache struct all land on metrics the gate itself does not touch.
