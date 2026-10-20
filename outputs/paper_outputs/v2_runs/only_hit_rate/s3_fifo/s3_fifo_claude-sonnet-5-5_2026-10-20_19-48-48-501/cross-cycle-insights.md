## Recurring wins

- Sizing is the only lever with consistent signal: a small queue shrunk from baseline (to roughly 2-5%) paired with a shortened ghost beats baseline. Gains arrive as small steps and flatten out.
- Sizing-only configurations match or beat more complex bundles. The ghosting of main evictees was not needed for the gain, so prefer the simplest config.
- Ghost length acts as an admission filter: shorter helps or is neutral, and longer has hurt in each test.
- Consolidating earlier gains into one parent gave the cleanest wins. Single-rule tweaks mostly gave ties or small losses.

## Recurring failures

- Loosening admission (looser promotion, pre-protected or boosted ghost re-admits, longer ghost) hurts by polluting main.
- Ageing and decay changes (faster freq decay, lower freq cap, halving, in-place ageing) have lost every time. The workload rewards keeping reuse history. A higher cap gave only a marginal gain.
- Rewrites of main eviction (lookahead, reinsertion caps) lose. The unbounded second-chance scan is protective.
- Filtering or restricting ghost hits or ghost contents (freq>0 gate, reuse-only ghosting) loses. Ghost hits are strong reuse evidence.
- Promotion strictness and timing (eviction-time gates, hit-path promotion, thresholds, freq reset) repeatedly tie exactly. They are either not binding on this trace or the diffs are no-ops.
- Adaptive or feedback-driven sizing lost to the fixed small ratio.
- Per-key ghost metadata and main insertion-position tweaks gave no signal.

## Open questions

- Do exact ties reflect real no-ops in the code? Variant authors cannot read overlay source, so this is unverified.
- Are the small gains (2% vs 3% vs 5%) real or noise? Sizing probes have never been compared head-to-head on one parent, and effective queue lengths may round to the same value.
- Do any results hold on a second trace or cache size? All evidence is one trace at one cache size.
- What do per-op-time and metadata-size look like? They have been unmeasured in every cycle, and ghost work and hit-path actions may add cost.
- Is there any structurally new lever beyond sizing, or is S3-FIFO at a local optimum here?

## Per-policy notes

- S3-FIFO (the only policy so far): the structure is near a local optimum on this trace. The best known base is the simple sizing-only config with a small queue of ~2% and a shortened ghost. Stop tuning ratio, ghost length, promotion thresholds, freq cap and ageing. Next steps are validation on a second trace and cache size, restoring cost metrics, and only structurally new levers whose diffs are verified to change behaviour.
