# Atomic ledger append

Every ledger write goes through filelock + atomic-rename in
`src/cycle/ledger_append.py`.

## Considered options

- **Per-variant ledger files merged at cycle seal.** Avoids contention but
  defers conflict-handling to a merge step and complicates readers (which
  would have to union multiple files). Rejected.
- **Plain append without locking.** Cheapest, but a cycle runs variants in
  parallel (P1 in-cycle parallelism) and any two concurrent writers can
  interleave bytes inside a JSONL row. Rejected.
- **Lock + atomic rename on every append.** Slightly more code, but makes
  concurrent variant execution within a cycle safe by construction. Chosen.

## Consequences

- The ledger append path is non-negotiable in code review — a non-atomic
  ledger write is a bug.
- Concurrent variant execution within a cycle is safe without further
  coordination on the ledger.
