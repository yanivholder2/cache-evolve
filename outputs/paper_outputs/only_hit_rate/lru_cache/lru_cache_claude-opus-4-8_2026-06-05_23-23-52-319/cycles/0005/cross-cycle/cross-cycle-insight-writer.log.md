# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-05T21:15:14+00:00

## timeline

### 21:15:20 thinking



### 21:15:34 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the dominant lever: every\n  win held capacity fixed and gained purely by choosing which resident\n  dies first.\n- \"Protect on second access\" is the core signal — distinguishing\n  one-hit-wonders from genuine reuse repeatedly converts directly to hits\n  on this reuse-heavy trace.\n- Enlarging the protected tier paid off across multiple consecutive\n  cycles (80/20 to 90/10 to 95/5), each step beating its parent — though\n  the marginal gain shrank every time.\n- Reuse on this trace arrives fast (within the narrow probationary\n  window), so first-hit promotion is confirmed correct.\n\n## Recurring failures\n\n- Loosening the promotion signal (wider probationary, two-hit gates)\n  consistently regresses: fast reuse means stricter/slower gates just\n  shed keys before their second touch.\n- Non-recency admission/victim signals (W-TinyLFU window, LFU victim\n  selection, frequency at demotion) keep losing inside the SLRU frame —\n  they mistrack recency or evict genuinely-reused keys; this trace is not\n  one-hit-wonder dominated.\n- Ghost / reuse-distance re-admission is directionally right but\n  structurally redundant against a maximally-protected tier, and costs\n  extra structure plus per-insert scans; tested in isolation and layered,\n  both inert.\n\n## Open questions\n\n- Has the protected-ratio lever hit its ceiling at 95/5, or is there a\n  sliver left before the probationary floor breaks the eviction path?\n- The canonical LIRS rebuild reported a real hit-rate lift but did not\n  land in the Winners — was the gain within noise, or did it trip a\n  contract/invariant check? Must be resolved before trusting it.\n- If the LIRS gain is real, does it dominate the SLRU incumbent outright\n  or only on this LIRS-derived home-turf trace?\n- Is any orthogonal workload signal left, or is the recency/promotion\n  axis fully consumed?\n\n## Per-policy notes\n\n- **Pure LRU**: baseline; loses by evicting reused keys as readily as\n  one-hit-wonders. Dropped early.\n- **SLRU (segmented)**: the durable parent across cycles. Binding\n  constraint is protected-tier residency (then the probationary floor),\n  not promotion timing or intra-segment order. Geometry now near-exhausted.\n- **CLOCK / midpoint insertion**: cheap single-list approximations of the\n  protection signal; helped initially but never beat explicit segmentation.\n- **LFU / frequency-based**: regresses on this trace at high protected\n  ratios — frequency values tie and it just mistracks recency.\n- **LIRS**: the one genuinely off-axis mechanism to report a real gain;\n  unresolved (reported lift did not win). The live thread to re-run cleanly.\n"
}
```

### 21:15:44 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the dominant lever: every\n  win held capacity fixed and gained purely by choosing which resident\n  dies first.\n- \"Protect on second access\" is the core signal — distinguishing\n  one-hit-wonders from genuine reuse repeatedly converts directly to hits\n  on this reuse-heavy trace.\n- Enlarging the protected tier paid off across multiple consecutive\n  cycles (80/20 to 90/10 to 95/5), each step beating its parent — though\n  the marginal gain shrank every time.\n- Reuse on this trace arrives fast (within the narrow probationary\n  window), so first-hit promotion is confirmed correct.\n\n## Recurring failures\n\n- Loosening the promotion signal (wider probationary, two-hit gates)\n  consistently regresses: fast reuse means stricter/slower gates just\n  shed keys before their second touch.\n- Non-recency admission/victim signals (W-TinyLFU window, LFU victim\n  selection, frequency at demotion) keep losing inside the SLRU frame —\n  they mistrack recency or evict genuinely-reused keys; this trace is not\n  one-hit-wonder dominated.\n- Ghost / reuse-distance re-admission is directionally right but\n  structurally redundant against a maximally-protected tier, and costs\n  extra structure plus per-insert scans; tested in isolation and layered,\n  both inert.\n\n## Open questions\n\n- Has the protected-ratio lever hit its ceiling at 95/5, or is there a\n  sliver left before the probationary floor breaks the eviction path?\n- The canonical LIRS rebuild reported a real hit-rate lift but did not\n  land in the Winners — was the gain within noise, or did it trip a\n  contract/invariant check? Must be resolved before trusting it.\n- If the LIRS gain is real, does it dominate the SLRU incumbent outright\n  or only on this LIRS-derived home-turf trace?\n- Is any orthogonal workload signal left, or is the recency/promotion\n  axis fully consumed?\n\n## Per-policy notes\n\n- **Pure LRU**: baseline; loses by evicting reused keys as readily as\n  one-hit-wonders. Dropped early.\n- **SLRU (segmented)**: the durable parent across cycles. Binding\n  constraint is protected-tier residency (then the probationary floor),\n  not promotion timing or intra-segment order. Geometry now near-exhausted.\n- **CLOCK / midpoint insertion**: cheap single-list approximations of the\n  protection signal; helped initially but never beat explicit segmentation.\n- **LFU / frequency-based**: regresses on this trace at high protected\n  ratios — frequency values tie and it just mistracks recency.\n- **LIRS**: the one genuinely off-axis mechanism to report a real gain;\n  unresolved (reported lift did not win). The live thread to re-run cleanly.\n"
}
```

### 21:15:50 chat

Wrote `cross-cycle-insights.md` — a fresh projection of all five cycles into the four fixed sections:

- **Recurring wins**: eviction ordering over residency, "protect on second access," the steady protected-tier enlargement (80/20→95/5), and fast-arriving reuse favoring first-hit promotion.
- **Recurring failures**: loosened promotion signals, non-recency admission/victim signals inside SLRU, and ghost re-admission (redundant against a maximal protected tier).
- **Open questions**: the 95/5 ceiling, the unresolved LIRS lift that didn't win, whether LIRS generalizes beyond its home trace, and whether any orthogonal signal remains.
- **Per-policy notes**: grouped observations for LRU, SLRU, CLOCK/midpoint, LFU/frequency, and LIRS.


---

- finished: 2026-06-05T21:15:50+00:00
- duration: 36.16s
- status: ok
- input_tokens: 5
- output_tokens: 2700
- cache_read_tokens: 62846
- cache_write_tokens: 10066
- cost_usd: 0.10087
