# v1 single-machine execution + five v2-ready invariants

The framework's ambition includes multi-machine parallel execution, but
building that for v1 is a semester of distributed-systems work (work queues,
atomic cross-machine ledger, worker-death recovery) that is not cache research
and that delays evidence of the framework's value. **v1 ships single-machine**
with P1 (variant-level within a cycle) + P2 (trace-level within a variant)
parallelism, bounded by local cores. **v2 multi-machine is deferred**, but four
invariants are baked into v1 so that v2 becomes a storage-layer swap rather
than a refactor:

1. **Repo-relative paths only** — no absolute paths in any artifact.
2. **Atomic ledger append** — filelock + atomic-rename in
   `src/ledger_append.py`, works locally and across NFS / shared blob in v2.
3. **Stable Variant fingerprint** — hash of `impl/` contents for cross-worker
   dedup and cross-cycle re-proposal detection.
4. **Insight ledger is the only synchronization point** — all other state is
   per-Cycle, per-Variant, write-once.

## Considered options

- **v1 multi-machine.** Technically possible but eats thesis budget on infra
  that is not the contribution. Rejected.
- **v1 single-machine without v2 invariants.** Cheapest now, but the v2
  retrofit becomes a months-long refactor that touches every artifact and
  script. Rejected.
- **v1 single-machine with v2-ready invariants.** Same v1 implementation cost
  as the no-invariants option (within ~10%), v2 becomes a storage-layer swap.
  Chosen.

## Consequences

- v1 scaling claim is honest: "scales to a single multi-core machine; the
  architecture is designed so multi-machine execution requires swapping the
  storage layer."
- The invariants are non-negotiable in v1 code review — any absolute path or
  non-atomic ledger write is a v1 bug.
- v2 work is bounded: replace the shared-filesystem ledger with a remote ledger
  service, add a work queue, add worker-death recovery. The framework itself
  does not change shape.
