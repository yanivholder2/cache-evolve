# Variant v003 Insight

## Helped

- (none) — no dimension improved over the parent.

## Hurt

- per-op-time: the `if self.tail == Some(idx)` MRU guard added a data-dependent branch to the hottest path in both `get` and `put`; whether an access lands on the tail varies trace-to-trace, so the branch mispredicts and its penalty outweighs the handful of link writes it skips — get and put p50 both rose.
- per-op-time: the guard worsened, not improved, the get p99 tail it was meant to fix — the extra compare-and-branch lengthens the slow path rather than shortening it.

## Takeaway

Short-circuiting the unlink/push_back surgery when the node is already MRU does not pay off here: the saved field writes are cheap and predictable, while the guarding branch is poorly predictable and lands on the critical path, so it costs more than it saves. Hit-rate and metadata-size stayed identical, confirming the guard is policy-neutral, but the per-op-time target moved the wrong way — a cheaper hot path is not worth a mispredicted branch.
