# Variant v001 Insight

## Helped

- hit-rate: Splitting the single recency list into a probationary and a protected segment lets keys that are read a second time survive a scan of one-hit keys, which is the pattern the LIRS workload-a trace exhibits.

## Hurt

- metadata-size: The per-node `in_protected` flag and the four segment head/tail pointers plus the `prot_len` counter add fixed bytes to each node and to the cache struct, with the node growth coming from padding a single bool up to the next alignment step.
- per-op-time: `get` now unlinks from one list and re-links into another, and may demote a protected head on every promotion, so both `get` and `put` do more pointer work than the single-list baseline.

## Takeaway

Reuse-based segmentation is what moves hit-rate here: the gain depends on the trace containing one-hit scans that would otherwise flush re-referenced keys, so it should be expected to help less on workloads without that pattern. The cost is paid on every operation rather than only on misses, so any further gain must come from keeping the promotion and demotion path cheap.
