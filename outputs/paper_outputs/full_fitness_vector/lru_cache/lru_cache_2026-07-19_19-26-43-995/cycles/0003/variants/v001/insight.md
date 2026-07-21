# Variant v001 Insight

## Helped

## Hurt

- **per-op-time**: The `idx == prot_tail` guard is evaluated on every protected-queue access — not just the ones that short-circuit — so the extra branch and second pointer read (`self.nodes[idx].prev`) add overhead on all protected hits; on workload-a, where the prot_tail slot is not the dominant access target, this branch cost exceeds the pointer-write savings, pushing both get p50 and put p99 upward.

## Takeaway

Adding an early-exit guard to a hot path is only profitable when the guard fires often enough to amortize its own check cost. The MRU-tail guard is correct and semantics-preserving (hit-rate and node size are unchanged), but workload-a does not have sufficient concentration of re-accesses on the single prot_tail entry for the bypass to pay off — the guard makes every non-tail protected hit slightly more expensive while only skipping the unlink+push_back pair for the rare tail case.
