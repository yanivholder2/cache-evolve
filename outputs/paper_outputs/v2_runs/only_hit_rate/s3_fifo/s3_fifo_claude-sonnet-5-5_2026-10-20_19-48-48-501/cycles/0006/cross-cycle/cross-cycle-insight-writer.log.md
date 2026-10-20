# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:08:26+00:00

## timeline

### 20:08:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission-side sizing is the only lever that has paid off: a smaller small queue and a shorter ghost queue won repeatedly. Each step gave less than the one before.\n- Capacity or ratio changes beat rule-level tweaks. The small/main split moved results more cleanly than any filter or aging change.\n- Consolidating earlier winners (small-queue shrink, shorter ghost, ghosting main evictees) gave the latest gain. It added no per-key state.\n- Adding ghost coverage without changing admission or protection rules is safe. At best it is marginally positive.\n\n## Recurring failures\n\n- Loosening admission loses (looser promotion, a longer ghost, pre-protected ghost re-admits). Weakly-reused keys in main displace useful residents.\n- Aging or decay changes to main always lose (faster freq decay, a lower freq cap, in-place aging, halving). This workload rewards keeping reuse history.\n- Rewriting main-eviction mechanics loses (lookahead, reinsertion caps). The unbounded second-chance scan does real protective work.\n- Filtering ghost hits hurts, and so does giving them extra credit (a grace pass, a boosted freq, per-key ghost flags). Plain ghost-hit admission is already well used.\n- Promotion-gate strictness and main insertion position had no measurable effect. They are weak or possibly no-op levers.\n- Gains and losses are small, and the tied or neutral results are ambiguous because variant authors could not read the overlay source.\n\n## Open questions\n\n- Which part of the latest winner carries the gain: ghosting main evictees, or the re-bundled sizing?\n- Do the gains hold on a second trace and cache size? All evidence so far is one trace at one size, so some effects may be noise.\n- Does a very small probation window hurt scan-heavy traces?\n- Were the tied variants actual no-ops? Future diffs should be checked to change behaviour.\n- Why are metadata-size and per-op-time still unmeasured after six cycles? Does extra ghost work on `put` cost meaningful time?\n\n## Per-policy notes\n\n- S3-FIFO on the LIRS trace looks near a local optimum. Sizing is saturated, and eviction and aging rewrites regress.\n- Current base: a very small small queue, a shorter ghost, and main evictees ghosted.\n- Untried or under-tested directions: stricter promotion with a verified behaviour change, other traces and cache sizes, and an ablation of main-evictee ghosting.\n- Cost dimensions (metadata, timing) have never been reported, so every cost claim is unverified.\n"
}
```

### 20:08:39 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output directory. It uses the four fixed sections in order and is projected only from the six-cycle ledger digest.

- **Recurring wins:** only admission-side sizing has paid off, meaning a smaller small queue and a shorter ghost, with shrinking gains. Capacity and ratio changes beat rule-level tweaks, and consolidating earlier winners gave the latest gain.
- **Recurring failures:** loosening admission, aging or decay changes, rewriting main eviction, and giving ghost hits extra credit or filtering them have all lost. Promotion-gate strictness and main insertion position showed no measurable effect.
- **Open questions:**
  - Which part of the latest winner carries the gain: ghosting main evictees or the re-bundled sizing.
  - Whether the results hold on a second trace and cache size.
  - Whether the tied variants changed behaviour at all.
  - Why metadata-size and per-op-time are still unmeasured.
- **Per-policy notes:** S3-FIFO on the LIRS trace looks near a local optimum. The current base is a very small small queue, a shorter ghost, and main evictees ghosted.

I kept the numbers qualitative, as the constraints require.


---

- finished: 2026-10-20T20:08:39+00:00
- duration: 13.29s
- status: ok
- input_tokens: 4
- output_tokens: 1436
- cache_read_tokens: 37741
- cache_write_tokens: 9293
- cost_usd: 0.055861
