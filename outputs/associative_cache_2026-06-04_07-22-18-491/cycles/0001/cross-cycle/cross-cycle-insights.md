# Cross-Cycle Insights

## Recurring wins

- **Activating latent data-model capacity**: The cycle-1 winner exploited a field (`lfu_counter`) already carried on every item at zero structural cost. Changes that surface pre-existing but unused information tend to be both low-risk and immediately fitness-positive.
- **Targeting cold-burst displacement**: Both hit-rate-improving variants addressed the same root failure mode — hot items evicted by cold-item bursts. This pattern is a reliable signal; approaches that counteract burst-driven eviction have shown gain without regression.

## Recurring failures

- **Memory-efficiency variants flying blind**: `metadata-size` was absent from fitness vectors across every variant that touched memory (stash fields in v003, `timestamp` removal in v002). Hypotheses targeting memory cost cannot be validated until this measurement gap is closed — results are structurally unverifiable, not just inconclusive.
- **Pure dead-field removal as a hypothesis**: Removing an unused field (v002, `timestamp`) produced no measurable fitness change. Structural clean-ups are worth landing as baseline hygiene but should not be framed as fitness hypotheses until the relevant dimension is actually measured.

## Open questions

- Is `metadata-size` wired into the evaluator for this subject at all, or does it only appear when a variant explicitly opts in? Clarifying this unblocks an entire class of memory-efficiency hypotheses.
- Do v001 (frequency-aware eviction) and v003 (second-chance stash) compose? They target complementary failure modes — reduced entry into the stash vs. recovery of items that slip through — and gains may compound rather than saturate.
- Does the stash's per-bucket memory overhead (two `Vec` fields, v003) offset its hit-rate benefit once `metadata-size` is measurable? The net trade-off is currently unknown.

## Per-policy notes

- **ClassicLRU** (the only policy explored so far): frequency weighting on eviction (v001) is a confirmed winner with no regressions; second-chance stash (v003) shows hit-rate gain but carries unmeasured memory cost; `timestamp` removal (v002) is a safe structural clean-up with no fitness impact. All three variants probed the same root weakness — susceptibility to cold-item burst eviction — from different angles.
