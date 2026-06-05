# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-06-05T20:48:19+00:00

## timeline

### 20:48:23 thinking



### 20:48:24 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 20:48:29 thinking



### 20:48:41 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra capacity, is where the gains live: every\n  win held capacity fixed and improved which resident key dies first.\n- \"Protect on second access\" is the durable axis — distinguishing\n  one-hit-wonders from genuine reuse, encoded as segmentation, beat pure\n  recency and seeded every subsequent winner.\n- Enlarging the protected tier keeps paying: each cycle that pushed\n  protected residency higher (segmentation, then 80/20, then 90/10) won,\n  confirming protected-tier room as the dominant lever on this trace.\n\n## Recurring failures\n\n- Loosening the promotion path consistently backfires — delaying\n  promotion to a second hit, widening the probationary segment, and\n  over-trusting a single post-eviction reuse all lost hits. Reuse arrives\n  fast and within a narrow probationary window; first-hit promotion is\n  confirmed correct.\n- Cleverer eviction-order machinery (frequency-windowed victim selection,\n  ghost / reuse-distance memory) is near-inert as a standalone lever:\n  second-access promotion already captures most reuse before eviction\n  order matters, leaving only a thin warm-but-unpromoted band to rescue.\n\n## Open questions\n\n- Where is the protected-ratio ceiling? Probing past 90/10 (e.g. 95/5)\n  must eventually break the eviction path once probationary nears empty.\n- Do the confirmed-marginal levers (frequency window, weak-protection\n  ghost) contribute anything when *layered on* the maximally-protected\n  winner rather than tested in isolation on the original parent?\n- With promotion and ratio both near-tuned, is the trace's recency\n  structure largely consumed — is there an untapped, non-recency workload\n  signal worth a fundamentally different lever?\n\n## Per-policy notes\n\n- SLRU (segmented): the through-line winner. Strong local optimum whose\n  bias is \"promote early, protect generously\"; gains come from shifting\n  the protected/probationary split toward protected.\n- Single-list approximations (midpoint insertion, CLOCK reference bit):\n  helped over pure LRU as cheap \"second access earns protection\" signals,\n  but explicit segmentation outscored them.\n- Ghost / reuse-distance lists: directionally right only when routed to\n  the protected-LRU end (weak protection); promoting ghost hits straight\n  to protected-MRU crowds the hot set and regresses. Recoverable headroom\n  is small either way.\n- Pure-recency LRU baselines: dominated on this reuse-heavy trace; retired\n  as probes.\n"
}
```

### 20:48:45 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the three-cycle ledger, with the four fixed sections:

- **Recurring wins** — eviction *ordering* over capacity, "protect on second access," and the steadily-paying push toward a larger protected tier.
- **Recurring failures** — loosening the promotion path (two-hit gates, wider probationary) always regresses; clever eviction-order machinery is near-inert standalone.
- **Open questions** — the protected-ratio ceiling, whether marginal levers help layered on the winner, and whether the trace's recency structure is exhausted.
- **Per-policy notes** — grouped across SLRU, single-list approximations, ghost lists, and pure LRU.


---

- finished: 2026-06-05T20:48:46+00:00
- duration: 26.90s
- status: ok
- input_tokens: 5
- output_tokens: 1468
- cache_read_tokens: 58000
- cache_write_tokens: 6989
- cost_usd: 0.068037
