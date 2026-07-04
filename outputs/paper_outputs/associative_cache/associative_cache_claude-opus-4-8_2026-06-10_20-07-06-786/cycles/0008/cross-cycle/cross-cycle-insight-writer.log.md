# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-10T21:21:17+00:00

## timeline

### 21:21:29 thinking



### 21:21:30 chat

I'll write the cross-cycle insights as a fresh projection of the 8-cycle ledger.

### 21:21:30 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 21:21:33 thinking



### 21:21:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Explicit **segmentation** beats tuning any single frequency scalar:\n  separating a probationary tier from a protected tier, with promotion\n  gated on demonstrated reuse, is the strongest scan-resistant scheme found\n  and has anchored the baseline since it appeared.\n- **FIFO / oldest-first probation** outperforms newest-first churn: giving a\n  fresh key a full lap to earn promotion recovers rising keys that would\n  otherwise be discarded before their second reference.\n- **Durable protection (\"once-hot immortality\")** is load-bearing on this\n  trace. Every scheme that keeps proven-hot keys protected indefinitely, at\n  the current protected size, has won or held; every scheme that makes\n  protection harder to earn or easier to lose has regressed.\n- **Promote-straight-to-MRU** placement is settled; seating promotions lower\n  in the protected segment regresses.\n- A single **capacity-conserving geometry change** (halving bucket count,\n  doubling bucket size) lifted hit-rate via associativity alone — the one\n  hit-rate lever found outside admission/promotion/eviction logic.\n- Behavior-neutral **O(1) / reorder-eliding cost rewrites** are always safe to\n  adopt; they never harm hit-rate.\n\n## Recurring failures\n\n- **Any erosion of earned protection** regresses, repeatedly and for the same\n  reason (collision-bound trace): counter aging, front-admission composition,\n  two-hit promotion thresholds, protected caps with demotion, elite/third\n  tiers, and reserved-probationary slots have all failed.\n- **Recency approximation** (CLOCK one-bit reference) underperforms full\n  positional ranking in small buckets.\n- **Re-entry / re-signal shortcuts** weaken discrimination: ghost-ring fast\n  re-entry and LFU-min eviction fallback both let weakly-warranted keys seize\n  protected space and regressed.\n- **Geometry past one halving** overshoots in both directions — a second\n  halving scatters hot keys; an absolute associativity target degenerates to a\n  narrower fixed width. One halving is bracketed and optimal.\n- **Routing/hash-quality** improvements buy nothing — the default hash is\n  already effectively uniform on these keys.\n- The orthogonal Cycle-0001 levers (frequency victim-choice + front admission)\n  **do not compose** — they conflict.\n\n## Open questions\n\n- Is hit-rate now at its ceiling? Admission, promotion, eviction, re-entry,\n  protected sizing/tiering, routing, and bucket width have each been ruled out.\n- Should a **per-op-time fitness axis** be added? Cost-side wins keep recurring\n  and keep landing as undifferentiated ties, invisible to hit-rate-only\n  selection.\n- Is there an untested **trace-structure / workload-regime** axis (e.g. a\n  shifting working set) where staleness would dominate, or is this trace firmly\n  collision-bound?\n- Is the protected-segment cap permanently dead code on this trace, or does any\n  regime saturate it?\n\n## Per-policy notes\n\n- **Segmented-LRU (current lineage):** the durable baseline. Strongest on\n  scan resistance; saturated on hit-rate — admission, promotion, and eviction\n  logic all repeatedly probed without gain.\n- **FIFO probation:** the winning probation order; oldest-first promotion\n  window beats newest-first eviction.\n- **Pure LFU / frequency:** useful only as the original victim signal; aging,\n  LFU-min fallback, and letting `==1` newcomers into the tally all erode the\n  hot/rising distinction and regress.\n- **LRU (plain):** protected-end recency ranking beats banked frequency and\n  coarse one-bit (CLOCK) approximations for choosing which protected key to\n  drop.\n- **Geometry/associativity:** independent live axis; exactly one\n  capacity-conserving halving is optimal, bracketed on both sides.\n"
}
```

### 21:21:56 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger, using the four fixed sections. Key synthesis across cycles:

- **Recurring wins**: explicit segmentation, FIFO/oldest-first probation, durable "once-hot immortality" protection, promote-to-MRU placement, the single capacity-conserving geometry halving, and safe behavior-neutral cost rewrites.
- **Recurring failures**: every protection-erosion variant (aging, two-hit thresholds, caps/demotion, elite tiers, reserved slots), recency approximation, re-entry shortcuts, geometry past one halving, and routing/hash tuning — plus the non-composing Cycle-0001 levers.
- **Open questions**: whether hit-rate is at ceiling, whether a per-op-time fitness axis is warranted, and whether any trace-structure regime makes staleness relevant.
- **Per-policy notes**: grouped observations for segmented-LRU, FIFO probation, LFU/frequency, plain LRU, and geometry/associativity.


---

- finished: 2026-06-10T21:21:56+00:00
- duration: 39.45s
- status: ok
- input_tokens: 5
- output_tokens: 2881
- cache_read_tokens: 67100
- cache_write_tokens: 11837
- cost_usd: 0.113715
