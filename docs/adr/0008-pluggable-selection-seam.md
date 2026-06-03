# Winner selection is a pluggable seam, not a hardcoded Pareto front

Selecting a Cycle's **Winners** from its fitness vectors was hardcoded to the
Pareto front in two places (cohort-local and cross-cycle). We make selection a
**pluggable seam**: a `Selector` resolved by name from `SELECTOR_REGISTRY`,
mirroring `evolve.llm.ADAPTER_REGISTRY`. Two methods ship as built-ins:

- **`pareto`** — the Pareto front (current behavior). Scale-invariant, so it
  needs no normalization and computes no scalar score.
- **`weighted`** — the top-`winners_count` variants by `Σ wᵢ·normᵢ`, where each
  dimension is min–max normalized across the cohort after the direction flip
  that makes higher always better.

Each selector validates its own method-specific config in `from_config` and
exposes a method-blind `select(fitness_vectors) -> SelectionResult`. The call
sites (`compute_cycle_winners`, `compute_cross_cycle_winners`) never branch on
the method.

Alongside this, **fitness-vector composition** moves from manifest-implicit to an
explicit config knob. Capability (which dimensions a subject *can* measure) stays
in the Subject manifest (`[bench]`, `[metadata]`); activation moves to
`[fitness] dimensions` in `config.toml`. The runner measures only active
dimensions.

## The return contract

`SelectionResult(winners: list[str], scores: dict[str, float])`.

- `winners` stays `list[str]` (the front for `pareto`, the top-k for `weighted`)
  so downstream shape is unchanged.
- `scores` maps **every** variant to its scalar. `pareto` returns `{}` rather
  than fabricate a scalar it doesn't compute — consumers (leaderboard) must
  tolerate empty scores, which is why the leaderboard keeps an explicit Winner
  marker rather than inferring Winners from the score column.

Scores are persisted only in `cycle-winners.json` and `cross-cycle-winners.json`
and rendered in the leaderboard. They are deliberately kept out of the ledger row
and LLM prompts: a cohort-relative scalar would over-anchor agents that should
reason over the raw per-dimension summaries.

## Considered options

### The seam

- **Keep the hardcoded Pareto front.** Simplest, but the project needs
  weighted/single-objective selection for workload-specialization runs. Rejected.
- **A two-way `if method == "pareto"` switch.** Less ceremony than a registry,
  but doesn't match the established `ADAPTER_REGISTRY` precedent and bakes the
  method set into the call sites. Rejected.
- **Pluggable registry of `Selector` classes.** Chosen — uniform with the LLM
  adapter seam, resolved against a config string, errors on an unknown key.

### Normalization for `weighted`

- **Min–max across the cohort.** Chosen. Maps each dimension to `[0, 1]` so a
  weight means the same thing on every axis regardless of units. A degenerate
  dimension (all variants equal) contributes `0` rather than dividing by a zero
  spread.
- **z-score.** Rejected — sensitive to outliers and yields unbounded values that
  make weights hard to reason about.
- **Baseline-relative (ratio to the baseline anchor).** Rejected — couples the
  score to a single anchor and explodes when a baseline dimension is near zero.
- **No normalization (weight raw summaries).** Rejected — units differ per
  dimension (hit-rate ∈ [0,1] vs nanoseconds), so weights would be meaningless.

The accepted tradeoff: min–max scores are **cohort/union-relative**. Cross-cycle
weighted scores shift as the union grows each cycle, and cross-cycle selection
becomes "top-k best-so-far" rather than an exhaustive global front.

## Consequences

- A second computation site (cross-cycle) routes through the same seam; both call
  `selector.select(...)`.
- The baseline anchor may measure a **superset** of the active dimensions; the
  selector projects every vector onto the active dimensions before selecting, so
  baseline and variants compare cleanly.
- Validation splits by concern: config-internal checks (method ∈ registry,
  `winners_count` ≥ 1, non-empty `dimensions`, weight↔dimension match, weights
  absent under `pareto`) live in `config.py` / `Selector.from_config`. The
  cross-cutting check (active dimensions are subject-capable and ⊆ the baseline
  anchor) lives at the `cli.py` boundary, keeping `config.py` subject-agnostic.
- `winners_count` is shared by both stages and clamped to the population;
  `pareto` ignores it.
