## Recurring wins

- Admission-side sizing is the only lever that has reliably paid off: a much smaller small queue (~5% of capacity) and a shorter ghost. Gains were modest and flattened after the first steps.
- Capacity/ratio changes beat rule-level tweaks. The small/main split moved results more cleanly than any filter or aging rule.
- Simpler configurations match complex ones. Sizing alone (no main-evictee ghosting) equals the more elaborate bundle, so prefer the simplest base.
- Ghost length acts as an admission filter: shorter (stricter) helped, longer hurt.

## Recurring failures

- Loosening admission (looser promotion gate, longer ghost, pre-protected ghost re-admits) adds pollution and loses.
- Ageing and decay changes (faster freq halving, lower freq cap, in-place aging) lose every time. The workload rewards keeping reuse history.
- Rewriting main-eviction mechanics (lookahead, reinsertion caps) loses. Unbounded second-chance reinsertion does real protective work.
- Extra credit for ghost hits (grace pass, boosted freq, per-key ghost flags) loses slightly. Filtering ghost hits loses clearly.
- Promotion-threshold and insertion-position tweaks tie their parent repeatedly. These are probably non-binding levers or no-ops, so they give no evidence.
- Filtering which main evictees enter the ghost loses slightly.

## Open questions

- Do the sizing gains hold on a second trace and cache size? All evidence so far is one trace at one size, and the gains are small enough to be noise.
- Do the tied variants (promotion gate, insertion position) change behaviour at all? Does freq ever reach the gate value while an item sits in small?
- Does main-evictee ghosting add anything on top of sizing, or is it neutral?
- Does a very small probation window weaken filtering on scan-heavy traces?
- Metadata-size and per-op-time have never been measured, so cost claims are unverified. Can they be restored?
- Is S3-FIFO at a local optimum on this trace, so that only structurally new levers can help?

## Per-policy notes

- S3-FIFO (small/main/ghost): the current best base is the sizing-only configuration (small queue ~5%, short ghost, plain main eviction with reinsertion). Keep the ghost-hit fast path permissive. Remaining tunables (ratios, ghost length, promotion gate, ghost content) look saturated, and the last several cycles' best variants were ties or marginal gains.
- Next steps should be verified to change behaviour, validated on another trace, and cost-measured before any further tuning.
