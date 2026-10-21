# Variant v002 Insight

## Helped

- metadata-size: unchanged; the new `push_front` helper reuses the existing u32 links and segment bit, so no per-node or per-cache bytes were added.

## Hurt

- hit-rate: slightly lower than the parent. Placing a demoted protected node at the LRU end of probation makes it the next eviction, so entries that had proven reuse are dropped before they can be re-referenced. The scan-heavy trace evidently has reuse among those entries.
- per-op-time: p99 for get and put crept up a little, probably measurement noise. `push_front` does the same pointer work as `push_back`, so no real extra cost is expected.

## Takeaway

A demoted protected node is not the same as a cold one-shot entry. It has already shown reuse, and giving it a fresh probationary lifetime at the MRU end was better than making it the immediate eviction candidate. The change only affects the rare overflow-demotion path, so even in the best case it could move the result very little, and here it moved it slightly the wrong way.
