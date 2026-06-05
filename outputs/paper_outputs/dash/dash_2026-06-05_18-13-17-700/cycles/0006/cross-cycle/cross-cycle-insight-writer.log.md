# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:20:52+00:00

## timeline

### 19:21:00 thinking



### 19:21:00 chat

I'll write the cross-cycle insights as a fresh projection of the six-cycle ledger digest.

### 19:21:15 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Reallocating fixed budget toward the promotion-protected normal tier was the\n  strongest early lever: shrinking the over-provisioned stash and pushing the\n  split from baseline → even → normal-dominant climbed hit-rate for two cycles\n  before saturating.\n- Protecting the warm working set pays off repeatedly, whichever mechanism: a\n  selective stash gate, MRU insertion, or capacity all moved the same way.\n- Eviction/demotion-path dynamics are a genuinely independent seam from the\n  hit-path levers: landing demoted keys at MRU end gained where hit-path\n  position had already flattened.\n- Two-speed promotion (slow stash→target hop, fast target→probing hop) reserves\n  the protected tier for twice-referenced keys and gains where uniform\n  promotion flattened.\n- Stash dwell-time is the most durable source of gains: giving new cold keys\n  time to prove a second reference helps, and a cuckoo-style relocation hop into\n  an alternate bucket is the freshest positive seam late in the search.\n\n## Recurring failures\n\n- Removing or gutting stash admission regresses sharply — the filter is\n  load-bearing warm-set protection, not dead probation. Confirmed twice.\n- Front-inserting cold keys into the stash makes them the immediate eviction\n  victim and denies dwell time; regressed in two separate cycles.\n- Promotion-gate widening is exhausted from every angle — rank fraction\n  (quarter, half), len-2 loosening, and density-independent touch counting all\n  hit the same small-bucket occupancy ceiling and return slivers or negatives.\n- Swapping the protected tier to LFU eviction regressed: it retains\n  stale-but-frequent keys over the live working set.\n- Trading bucket count for bucket width regressed; addressability beats depth at\n  conserved budget — bucket count is a value to preserve.\n- Stacking a selective-admission gate on top of the capacity reshape was\n  redundant, not additive — the two chase the same imbalance.\n\n## Open questions\n\n- Is the recurring small-bucket occupancy ceiling a structural property of the\n  trace's hashing, or is there a path to denser target buckets that would\n  reopen the promotion-side levers?\n- Does the stash relocation hop deepen (multi-hop cascade, better victim\n  selection, larger alternate set) or saturate as fast as prior levers once\n  alternate buckets fill?\n- Are the late slivers additive or do they chase the same boundary-oscillating\n  key pool (gate width + demotion-MRU; relocation + two-speed promotion)?\n- Validation against a second, less-skewed trace is now overdue — levers have\n  flattened for several cycles, and the diminishing returns may be a property of\n  this single high-skew workload rather than the design.\n\n## Per-policy notes\n\n- Recency / promote-on-hit ranking suits the high-skew trace well; fast re-touch\n  means \"earn your way up\" delays cost few hits, so promotion-position tweaks\n  yield only slivers.\n- LFU on the protected tier is a dead lever on this workload — it holds stale\n  frequent keys against the oscillating live set.\n- Stash admission is settled and protective: dwell time is its value; its insert\n  position is closed in both directions (protective MRU default holds,\n  cold-front-insertion hurts).\n- Tier geometry (split ratio, promotion position, bucket count) is near-saturated;\n  the live frontier is stash dwell-time and eviction/demotion dynamics.\n"
}
```

### 19:21:21 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the six-cycle ledger. Summary of what each section captures:

- **Recurring wins** — protected-tier capacity reallocation (the dominant early lever, now saturated), warm-set protection generally, demotion-path MRU, two-speed promotion, and stash dwell-time/relocation as the freshest live seam.
- **Recurring failures** — removing/front-loading stash admission, promotion-gate widening (dead from every signal shape against the occupancy ceiling), LFU swap, bucket-widening, and redundant gate-stacking.
- **Open questions** — whether the small-bucket ceiling is structural, whether relocation deepens or saturates, lever additivity, and the overdue second-trace validation.
- **Per-policy notes** — recency/promote-on-hit fits the skew; LFU is dead here; stash admission settled/protective; geometry near-saturated with dwell-time/demotion as the frontier.

Kept it short and qualitative for downstream hypothesis-writer cycles.


---

- finished: 2026-06-05T19:21:21+00:00
- duration: 29.45s
- status: ok
- input_tokens: 3
- output_tokens: 2118
- cache_read_tokens: 38884
- cache_write_tokens: 8872
- cost_usd: 0.081226
