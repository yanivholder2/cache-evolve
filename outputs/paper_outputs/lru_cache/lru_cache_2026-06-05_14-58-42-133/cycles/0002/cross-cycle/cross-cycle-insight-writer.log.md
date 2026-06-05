# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-06-05T15:20:03+00:00

## timeline

### 15:20:07 thinking



### 15:20:17 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Separating \"seen once\" from \"proven,\" and protecting proven keys from\n  transient/scan traffic, has raised hit-rate in every cohort so far.\n- The promotion gate — promoting only on a genuine second reference — is\n  the load-bearing lever across cycles; winners respect it.\n- Letting the recency/frequency balance self-tune from ghost-hit signal\n  beat hand-set fixed structure, despite extra bookkeeping.\n- Ghost memory pays off when it *feeds* the promotion decision rather\n  than overriding it.\n\n## Recurring failures\n\n- Bypassing the promotion gate keeps backfiring: readmitting a returning\n  ghost key straight into the protected segment over-promotes low-reuse\n  keys, displaces hot ones, and regresses hit-rate.\n- Layering more structure (ghost queue) onto a fixed-split base without\n  improving the admission decision did not pay off.\n\n## Open questions\n\n- A frequency-sketch / TinyLFU admission gate on top of the adaptive\n  winner remains untested (its variants have not produced results).\n- All scoring is hit-rate only, so per-access overhead from extra lists,\n  ghost maps, and sketches is invisible — would the adaptive winner's\n  edge survive if cost were scored?\n- Do orthogonal signals (frequency + segmentation/adaptivity) compound,\n  or just add overhead?\n\n## Per-policy notes\n\n- Flat LRU: vulnerable to one-hit-wonder / scan pollution; the baseline\n  all variants improve on. Low overhead, but no scored dimension yet\n  rewards that.\n- SLRU (fixed split): a solid first base, but superseded; weak as a\n  standalone parent unless overhead becomes scored.\n- 2Q / TinyLFU-style admission: classic angles that improve over flat\n  LRU but have not won; the frequency-gate idea is still unevaluated.\n- ARC (adaptive): current proven base — self-tuning target driven by\n  ghost hits, respects the second-touch promotion rule, carries the most\n  bookkeeping.\n"
}
```

### 15:20:22 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the two-cycle ledger, using the four required sections in order.

Key threads carried forward:
- **Wins**: separating "seen once" from "proven," the second-reference promotion gate as the load-bearing lever, self-tuning beating fixed structure, and ghost memory helping only when it feeds (not overrides) promotion.
- **Failures**: bypassing the gate via direct-to-protected ghost readmission; layering structure without improving admission.
- **Open questions**: the still-untested frequency-sketch gate, invisible overhead under hit-rate-only scoring, and whether orthogonal signals compound.
- **Per-policy notes**: grouped observations for flat LRU, fixed SLRU, 2Q/TinyLFU admission, and the current ARC base.


---

- finished: 2026-06-05T15:20:22+00:00
- duration: 19.03s
- status: ok
- input_tokens: 3
- output_tokens: 1225
- cache_read_tokens: 35008
- cache_write_tokens: 4180
- cost_usd: 0.046157
