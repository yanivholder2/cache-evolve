## Recurring wins

- Orthogonal-axis changes both land: segmentation (hit-rate, scan resistance) and fast hasher/cheap touch (median latency, struct size) improved different axes and look combinable.
- Policy-level changes (segmented LRU) deliver hit-rate gains on scan-heavy traces; this is established and needs no more proof on that workload.
- Shrinking per-entry metadata and hot-path work pays off in median latency.

## Recurring failures

- Median gains have been bought with worse tail latency (p99): list surgery on promotion/demotion hurt put p99; weak hashing hurt get p99.
- Policy changes tend to grow metadata (bigger nodes, extra bookkeeping).
- Infrastructure failures (Maven download rate limit, HTTP 429) can reject a variant before evaluation. That says nothing about the idea's merit, so don't count it as a failed approach.
- Partial landings leave gains unused (the eviction-path key clone in put remained).

## Open questions

- Can the hash be fast without the get p99 regression (better mixing/fold)?
- Can segmentation's metadata cost be recovered (segment bit in existing fields, compact links)?
- Does the segmentation hit-rate gain hold on non-scan traces?
- Does u32 sentinel-link (compact arena) deliver its locality and size benefit? It is still untested.
- Does combining segmentation, fast touch and the better hasher compose additively?

## Per-policy notes

- LRU (baseline/arena): u32 compact links are untested because of the infra failure. Retry them. Removing the eviction-path key clone is still pending.
- Segmented LRU: hit-rate up on scan-heavy traces; costs put p99 and metadata.
- Fx-hash/fast-touch LRU: median latency and struct size improved, hit-rate unchanged; get p99 regressed.
