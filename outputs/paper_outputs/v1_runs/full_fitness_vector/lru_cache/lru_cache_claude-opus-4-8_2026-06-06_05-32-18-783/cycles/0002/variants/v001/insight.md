# Variant v001 Insight

## Helped

- metadata-size: encoding `prev`/`next` (and `head`/`tail`) as bare `u32` sentinels instead of no-niche `Option<usize>` drops each link from 16 to 4 bytes, halving `Node<u64,u64>` and shrinking the `LruCache` struct alongside it.

## Hurt

- per-op-time: get and put p50 both regressed versus v003 — the `NIL`-sentinel branch in the `idx` helper plus the per-mutation `link` narrowing add work to `unlink`/`push_back` that the previous `Option<usize>` niche representation handled for free.
- per-op-time: get and put p99 also moved up, so the tail cost of the sentinel encoding is real, not just a median effect.

## Takeaway

Re-encoding the recency links as `u32` sentinels does land the intended 24-byte node, matching v002's footprint on a pure representation change with hit-rate held byte-for-byte. Centralising the casts in `idx`/`link` keeps the slowdown well short of v002's ~78 ns blow-up, but it does not reach the hypothesis's goal of staying at v003's per-op-time: trading the `Option` niche for explicit sentinel checks still costs measurable median and tail latency.
