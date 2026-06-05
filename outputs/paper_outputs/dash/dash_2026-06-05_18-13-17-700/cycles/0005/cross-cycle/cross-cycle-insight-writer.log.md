# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:06:17+00:00

## timeline

### 19:06:22 thinking



### 19:06:23 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working dir and contents"
}
```

### 19:06:27 thinking



### 19:06:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting the warm working set in the promotion-protected normal tier is\n  the dominant source of hit-rate. Reallocating fixed budget away from the\n  over-provisioned stash and into the protected tier paid off repeatedly and\n  was the largest single lever found.\n- Eviction/demotion-path dynamics are a separate, still-live seam from the\n  hit-path mechanics: re-landing demoted keys at MRU protects the\n  oscillating working set and gains where promotion-position levers had\n  already flattened.\n- Two-speed promotion (slow first hop into target, fast second hop into\n  probing) reserves the protected tier for twice-touched keys and blocks\n  one-hit cold spikes — a genuinely distinct working lever.\n- Every win has been budget-conserving and invariant-preserving; gains come\n  from reshaping where budget sits, not from spending more of it.\n\n## Recurring failures\n\n- Removing or gutting stash admission regresses sharply (shown twice). The\n  stash filter is load-bearing: it gives new keys dwell time to prove\n  themselves, and without it cold one-hit-wonders displace resident hot keys.\n- Stacking a selective-admission frequency gate on top of an already-enlarged\n  protected tier is redundant — most benefit is captured by tier capacity, not\n  by extra admission filtering.\n- Front-inserting cold admissions (differentiating them from warm demotions\n  by stash position) regresses: it makes new keys the immediate eviction\n  victim and denies the dwell time the stash exists to provide.\n- Trading bucket count for bucket width regresses — addressability beats\n  depth; hot keys collide into fewer landing spots.\n- Pushing the normal/stash split past its peak no longer climbs; the\n  capacity-reallocation axis is saturated.\n\n## Open questions\n\n- Both surviving promotion-side levers (fractional gate, two-speed promotion)\n  are throttled by the same root cause: target buckets skew to len-1/len-2, so\n  no rank/recency rule has daylight. Is there any path to denser target\n  buckets, or does the trace's hashing structurally cap every promotion lever?\n- Are the freshest winning levers (demotion-MRU, widened gate, two-speed\n  promotion) additive, or do they all chase the same thin pool of\n  boundary-oscillating keys?\n- Has the cohort over-fit to a single high-skew trace? The flattening of\n  nearly every lever may be a property of this workload — validation against a\n  second, less-skewed trace is overdue before chasing further slivers.\n\n## Per-policy notes\n\n- DASH tiering (stash + promotion-protected normal tier): the consistent\n  winning structure. Budget belongs in the protected tier; the stash should be\n  selective but never removed.\n- Recency / promote-on-hit on the protected tier: well-matched to the skewed\n  trace and the basis of every gain.\n- LFU eviction on the protected tier: a dead lever — it retained\n  stale-but-frequent keys over the live working set and regressed.\n"
}
```

### 19:06:45 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 5-cycle ledger. Summary of what it captures:

- **Recurring wins**: protected-tier capacity reallocation (the dominant lever), demotion-path MRU landing, and two-speed promotion — all budget-conserving.
- **Recurring failures**: removing/gutting stash admission (regresses twice), stacking redundant frequency gates, front-inserting cold keys, trading bucket count for width, and over-pushing the split past saturation.
- **Open questions**: the small-bucket occupancy ceiling on promotion levers, additivity of the fresh levers, and overdue validation on a less-skewed trace.
- **Per-policy notes**: grouped by DASH tiering, recency/promote-on-hit (works), and LFU eviction (dead lever).


---

- finished: 2026-06-05T19:06:46+00:00
- duration: 28.44s
- status: ok
- input_tokens: 5
- output_tokens: 1610
- cache_read_tokens: 60196
- cache_write_tokens: 7610
- cost_usd: 0.074581
