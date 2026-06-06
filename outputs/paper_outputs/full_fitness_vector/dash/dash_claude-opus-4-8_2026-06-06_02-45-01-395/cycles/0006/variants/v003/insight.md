# Variant v003 Insight

## Helped

- metadata-size: Carrying heat as a per-bucket `Vec<u8>` in the bucket container (never in `Item`) kept `Item<u64,u64>` at the 16-byte floor exactly as the hypothesis required — the structural move added no per-entry layout.
- per-op-time: Deleting the `target_heat` HashMap removed its `entry`/`remove` traffic from the put path, which shaved the put p99 tail down — the one place removing the hash showed through.

## Hurt

- per-op-time: get-p50 and get-p99 both rose sharply because `get_from_position` now does an extra O(n) `heat.remove(position); heat.push(h)` Vec shift on *every* ClassicLRU reorder to keep heat aligned with its item — a per-hit array cost broader than the parent's target-only hashed probe, and put-p50 edged up for the same reason.
- hit-rate: dipped one tick because the parallel-vector alignment plus implicit "reset on re-insert" did not reproduce the parent's promotion-gate decisions bit-for-bit, nudging a few promotions off.

## Takeaway

Removing the side-table hash did not buy latency: the heat must ride along with its item through ClassicLRU's remove+push, so the parallel `Vec<u8>` pays an O(n) element shift on every get — paid on all hits, not just target hits like the parent's probe — making the hot path slower, not faster. The hash probe was never the bottleneck; maintaining a position-aligned array under reorder is more expensive than the hash it replaced, and the alignment also failed to preserve gate semantics exactly.
