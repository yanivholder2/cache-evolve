# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003
- allow_edits: True
- started: 2026-06-05T23:25:17+00:00

## timeline

### 23:25:20 thinking

Let me read the variant insights and hypotheses for cycle 0003 to understand what was tried and what won.

### 23:25:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v001/insight.md"
}
```

### 23:25:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v002/insight.md"
}
```

### 23:25:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v003/insight.md"
}
```

### 23:25:24 thinking

Now let me read the hypotheses for context.

### 23:25:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/hypotheses/H01.md"
}
```

### 23:25:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/hypotheses/H02.md"
}
```

### 23:25:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/hypotheses/H03.md"
}
```

### 23:25:45 thinking

Now I have all the information I need. Let me synthesize this into a cycle insight.

**Summary:**
- **v001** (H01): Bounded frequency scan (k=8) to pick the coldest item for demotion from protected. Result: negligible improvement. The protected segment at 90% ratio is already well-packed with hot items; the scan rarely finds a meaningfully colder candidate than the LRU head.
- **v002** (H02): Ghost set + 80/20 SLRU from scratch. Result: small positive gain, no regressions. The ghost mechanism provides fast-track re-admission for recently evicted items, but the small ghost buffer means items cycle out before being re-requested.


- **v003** (H03, **WINNER**): High promotion threshold (PROMO_THRESHOLD=8) + `was_protected` bypass flag. Result: small real gain. The threshold blocks borderline items from polluting protected; the bypass allows truly hot demoted items to return immediately.

Looking at the patterns across these experiments, both the frequency-guided demotion scan and the promotion gating targeted the protected segment's quality but from opposite directions—one by improving what gets demoted, the other by filtering what gets promoted. Only the promotion-side intervention actually moved the needle, while the demotion-side barely registered. The ghost set introduced a structural mechanism that delivered real gains without any regressions, though the small buffer limited its effectiveness.

The `was_protected` bypass is conceptually crucial—without it, strict promotion gating would trap hot items cycling through overflow. All three variants stayed positive or neutral, suggesting we're near a local optimum where further progress likely needs structural changes rather than parameter tweaks. Demotion policy tuning within the protected segment seems mostly exhausted at the current ratio.

I'm wondering whether layering the ghost-set mechanism on top of v003's promotion gating would compound the gains or create interference, and whether a larger ghost buffer could unlock more frequent fast-track hits. The promotion threshold of 8 might be close to optimal, but I should explore whether 5 or 12 shift performance meaningfully, and whether tuning the admission-side frequency gate is where the remaining leverage sits.

For the next cycle, v003 becomes the baseline—the `was_protected` bypass paired with threshold-8 promotion is solid and worth keeping. I want to test the ghost-cache admission from v002 in isolation on top of v003 to see how much it contributes independently from the SLRU segmentation. The protected-segment demotion scanning from v001 looks exhausted at the 90/10 ratio, so I'll deprioritize that. Since gains are flattening, the next structural lever to investigate is either ghost-set sizing or admission-side tuning.

Now I'm writing up the cycle insights.

### 23:25:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/cycle-insight.md",
  "content": "# Cycle 0003 Insight\n\n## Cohort outcome\n\nAll three variants targeted the protected segment from different angles — frequency-guided demotion selection, ghost-set fast-track re-admission, and promotion gating with a demotion-history bypass. Only the promotion-gate variant (v003) was selected; all three produced neutral-to-positive results, reinforcing that the configuration is near a local optimum where incremental gains are small.\n\n## Patterns\n\n- **Promotion-side filtering beat demotion-side selection.** Replacing the scan-based demotion picker (v001) with a threshold gate on *promotion* (v003) was the only change that cleared the winner bar. At a 90/10 split the protected segment's LRU tail is already high-frequency, so searching for a colder demotion candidate rarely overrides the default LRU choice; blocking borderline items from entering protected in the first place yields more.\n- **The `was_protected` bypass is the essential partner to aggressive gating.** Raising the promotion threshold without distinguishing overflow-demoted items from frequency-weak newcomers would penalize genuinely hot items caught in capacity churn. The bypass makes the threshold safe to set high.\n- **Ghost-set re-admission is real but buffer-size-constrained.** The ghost-ring mechanism produced a positive gain with no regressions, but at a buffer capped near capacity/8 the fast-track path fires infrequently because evicted items cycle out of the ring before they are re-requested. The mechanism and the SLRU changes were introduced together, leaving their individual contributions unresolved.\n- **The cohort produced no regressions.** With three variants all landing flat-to-positive, the cache is operating well within a safe region, but the absence of a decisive winner signals diminishing returns from structural tuning of the protected segment alone.\n\n## Open questions\n\n- Does the ghost-set mechanism from v002 add further gain when layered on top of v003's promotion gating, or do the two interact and neutralize each other?\n- What ghost-buffer size (e.g., capacity/4 or capacity/2) is needed for the fast-track path to fire frequently enough to matter on this trace?\n- Are admission-side changes — adjusting the TinyLFU frequency gate itself — the remaining meaningful lever, given that protected-segment internals appear nearly exhausted?\n\n## Continuation\n\nTake v003's 90/10 SLRU with promotion threshold and `was_protected` bypass as the new baseline. The next cycle's highest-value experiment is isolating the ghost-set mechanism on top of v003 (not combined with a fresh SLRU variant) to determine whether ghost-set re-admission contributes independently. If that confirms a real gain, a follow-on probe on ghost-buffer size is warranted. Protected-segment demotion scanning should be deprioritized; admission-side or decay-side hypotheses are the better structural frontier if ghost-set isolation also disappoints.\n"
}
```

### 23:26:06 chat

Written `cycle-insight.md` to the output dir. Here's a summary of what it captures:

**Cohort outcome:** All three variants targeted the protected segment from different angles and all landed neutral-to-positive — a sign the cache is near a local optimum where gains are small.

**Key patterns identified:**
- Promotion-side filtering (v003's threshold gate) beat demotion-side selection (v001's frequency scan) — at the 90/10 split ratio, the LRU tail of protected is already high-frequency so there's little to gain by scanning for a colder demotion candidate.
- The `was_protected` bypass is load-bearing for the high threshold: without it, genuinely hot overflow-demoted items would be penalized by the same gate.
- The ghost-set mechanism (v002) showed real promise but is buffer-size-constrained, and its contribution is confounded by being introduced alongside a fresh SLRU structure.
- Zero regressions across the cohort confirms a safe but diminishing-returns zone.

**Continuation directive:** Isolate the ghost-set mechanism on top of v003's structure as the highest-priority next hypothesis; deprioritize demotion-scan tuning; consider admission-side changes as the next structural frontier if ghost-set isolation also disappoints.


---

- finished: 2026-06-05T23:26:06+00:00
- duration: 48.73s
- status: ok
- input_tokens: 5
- output_tokens: 2380
- cache_read_tokens: 60647
- cache_write_tokens: 8756
- cost_usd: 0.088866
