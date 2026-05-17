---
trace: zipf-medium
suite: holdout
hidden: true
generator: zipfian
seed: 201
events: 10000
key_universe: 800
signed_off_by:
---

# zipf-medium (HELD-OUT)

**HIDDEN FROM CONTEXT-BUNDLE ASSEMBLY.** Do not surface this profile or its
trace contents to the variant-implementer or hypothesis-proposer subagents.

Zipfian (α=0.95) over an 800-key universe — between `zipf-small` and
`zipf-large` in both skew and universe. A variant that over-fits to either
end of the dev Zipf pair should be visibly weaker here.

## Workload characteristics

- **Skew**: heavy-headed (α=0.95).
- **Working-set size**: 800 keys; hot-set ~80.
- **Scan behavior**: none.
- **Temporal locality**: high on the head, thin on the body.
