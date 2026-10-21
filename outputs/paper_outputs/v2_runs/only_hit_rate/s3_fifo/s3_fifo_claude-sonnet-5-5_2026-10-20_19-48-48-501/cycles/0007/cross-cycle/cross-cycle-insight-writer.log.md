# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:11:18+00:00

## timeline

### 20:11:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission-side sizing is the only lever that has reliably paid off: a much smaller small queue (~5% of capacity) and a shorter ghost. Gains were modest and flattened after the first steps.\n- Capacity/ratio changes beat rule-level tweaks. The small/main split moved results more cleanly than any filter or aging rule.\n- Simpler configurations match complex ones. Sizing alone (no main-evictee ghosting) equals the more elaborate bundle, so prefer the simplest base.\n- Ghost length acts as an admission filter: shorter (stricter) helped, longer hurt.\n\n## Recurring failures\n\n- Loosening admission (looser promotion gate, longer ghost, pre-protected ghost re-admits) adds pollution and loses.\n- Ageing and decay changes (faster freq halving, lower freq cap, in-place aging) lose every time. The workload rewards keeping reuse history.\n- Rewriting main-eviction mechanics (lookahead, reinsertion caps) loses. Unbounded second-chance reinsertion does real protective work.\n- Extra credit for ghost hits (grace pass, boosted freq, per-key ghost flags) loses slightly. Filtering ghost hits loses clearly.\n- Promotion-threshold and insertion-position tweaks tie their parent repeatedly. These are probably non-binding levers or no-ops, so they give no evidence.\n- Filtering which main evictees enter the ghost loses slightly.\n\n## Open questions\n\n- Do the sizing gains hold on a second trace and cache size? All evidence so far is one trace at one size, and the gains are small enough to be noise.\n- Do the tied variants (promotion gate, insertion position) change behaviour at all? Does freq ever reach the gate value while an item sits in small?\n- Does main-evictee ghosting add anything on top of sizing, or is it neutral?\n- Does a very small probation window weaken filtering on scan-heavy traces?\n- Metadata-size and per-op-time have never been measured, so cost claims are unverified. Can they be restored?\n- Is S3-FIFO at a local optimum on this trace, so that only structurally new levers can help?\n\n## Per-policy notes\n\n- S3-FIFO (small/main/ghost): the current best base is the sizing-only configuration (small queue ~5%, short ghost, plain main eviction with reinsertion). Keep the ghost-hit fast path permissive. Remaining tunables (ratios, ghost length, promotion gate, ghost content) look saturated, and the last several cycles' best variants were ties or marginal gains.\n- Next steps should be verified to change behaviour, validated on another trace, and cost-measured before any further tuning.\n"
}
```

### 20:11:33 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four fixed sections in order, built only from the seven-cycle ledger digest.

- **Recurring wins:** Admission sizing is the one lever that has paid off. That means a much smaller small queue (about 5% of capacity) and a shorter ghost. Capacity and ratio changes moved results more cleanly than rule-level tweaks. The sizing-only setup matches the more complex bundle, so the simpler one is preferred as the base.
- **Recurring failures:** Several approaches kept losing or tying:
  - Loosening admission lost.
  - Ageing and decay changes lost every time.
  - Rewrites of main eviction lost.
  - Extra credit for ghost hits lost slightly, and filtering ghost hits lost clearly.
  - Promotion-threshold and insertion-position tweaks only tied their parent. They may be no-ops, so they give no real evidence.
- **Open questions:**
  - Do the gains hold on a second trace and cache size? All evidence so far is from one trace at one size.
  - Do the tied variants actually change behaviour?
  - Does ghosting main evictees add anything on top of sizing?
  - Can metadata-size and per-op-time be measured? They have never been reported, so cost claims are unverified.
- **Per-policy notes:** The current S3-FIFO base is the sizing-only configuration. The remaining tunables look saturated. Further changes should be checked to change behaviour, validated on another trace, and cost-measured.

I used qualitative terms throughout and did not read the previous insights file.


---

- finished: 2026-10-20T20:11:33+00:00
- duration: 15.19s
- status: ok
- input_tokens: 4
- output_tokens: 1573
- cache_read_tokens: 38486
- cache_write_tokens: 10071
- cost_usd: 0.060493
