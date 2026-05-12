# Append-only Insight ledger + recomputable Synthesis

The evolve framework needs durable cross-cycle reasoning. A naive design uses a
single mutable synthesis document the AI updates each cycle, but that loses
history: the AI silently rewrites away prior observations and drifts under
recency bias, destroying the longitudinal record that is the whole point. We
split the record into two layers: an **append-only JSONL Insight ledger** (one
immutable row per Cycle, the source of truth) and a **recomputable Synthesis
view** the AI rewrites freely (derived from the ledger; never authoritative).
Atomic-append is enforced by the ledger-append script, never by the LLM.

## Considered options

- **Single mutable synthesis doc.** Simpler, but the AI rewriting it each cycle
  silently destroys history. Rejected.
- **Per-cycle insights only, no synthesis.** Preserves history but loses the
  narrative thread the framework needs to seed future Hypotheses. Rejected.
- **Append-only ledger + recomputable synthesis.** Two layers, durable
  source-of-truth plus disposable narrative. Chosen.

## Consequences

- All cross-cycle reasoning loads via a digest script over the ledger; the
  Synthesis view is convenience, not authority.
- If a Synthesis goes wrong, recompute from the ledger. If the ledger is
  corrupted, history is lost — atomic-append is non-negotiable.
- The split aligns with the token-frugal principle (ADR-0002): the LLM writes
  prose, the script enforces durability.
