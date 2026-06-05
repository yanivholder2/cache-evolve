# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-17-700/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:37:26+00:00

## timeline

### 19:37:41 thinking



### 19:37:46 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting the warm working set is the dominant theme: every early gain came\n  from giving the promotion-protected normal tier more room or shielding it\n  from transient keys (capacity reallocation, insertion position, promotion\n  gating).\n- Capacity reallocation toward the protected tier was the strongest single\n  lever — it paid off across two cycles past the even split before saturating.\n- Stash dwell-time is the deepest still-live seam: letting cold keys survive\n  long enough to catch a second reference keeps yielding clean, capacity-\n  conserving gains when promotion-side levers have flattened.\n- Eviction/demotion-path placement (landing demoted keys at MRU) is a genuinely\n  independent lever from hit-path placement and gained where the hit-path had\n  already exhausted.\n- Reactive rescue of real collision victims (multi-hop relocation cascade)\n  beats single-hop and beats proactive balancing — depth pays where breadth\n  does not.\n\n## Recurring failures\n\n- Promotion-gate widening is exhausted from every angle (rank, fraction,\n  density-independent touch counting): target buckets skew so hard toward\n  len-1/len-2 that no signal shape beats the small-bucket occupancy ceiling.\n- Removing or gutting stash admission regresses sharply — the stash filter is\n  load-bearing warm-set protection, not dead probation.\n- Front-landing cold stash keys (cold/warm differentiation by insert position)\n  regressed twice: cold keys need dwell time, not immediate eviction-victim\n  status.\n- Swapping the protected tier to LFU regressed — recency/promote-on-hit suits\n  the skew; LFU retains stale-but-frequent keys.\n- Trading bucket count for bucket width regressed — addressability beats depth;\n  bucket count is a value to preserve.\n- Pre-emptive load-balancing (power-of-two-choices admission) regressed —\n  primary-first locality matters more than even fill.\n- Independently-discovered gains tend to overlap rather than add: stacked levers\n  (gate+demotion, single-hop+protected-tier) came back flat, all chasing the\n  same thin pool of boundary-oscillating keys.\n\n## Open questions\n\n- Does the stash relocation cascade keep paying as it deepens (larger hop\n  budget, smarter-than-LRU victim selection), or does it saturate once the\n  reachable bucket neighborhood fills — the occupancy ceiling in a new guise?\n- Is there *any* pair of discovered seams that compounds, or is every lever\n  ultimately competing for the same small boundary-key pool?\n- Is the recurring skew/occupancy bottleneck a property of the design or of a\n  single high-skew trace? Validating against a second, less-skewed trace is now\n  long overdue before investing further.\n\n## Per-policy notes\n\n- Recency / promote-on-hit (protected normal tier): the right default for this\n  skew; benefits from more capacity and MRU demotion placement, but its\n  promotion-gate and split levers are now saturated.\n- LFU on the protected tier: a dead lever here — retains stale frequent keys\n  over the live working set and regresses.\n- Stash (admission/probation tier): the only consistently live axis; its value\n  is dwell time. Protective MRU default holds; relocation-cascade depth is the\n  freshest positive direction.\n"
}
```

### 19:37:51 chat

Wrote `cross-cycle-insights.md` — a fresh projection of all 7 ledger cycles into the four fixed sections:

- **Recurring wins**: warm-set protection (the dominant theme), capacity reallocation as the strongest early lever, stash dwell-time as the deepest still-live seam, independent demotion-path placement, and depth-over-breadth relocation.
- **Recurring failures**: promotion-gate widening (exhausted from all angles), removing/front-landing stash keys, LFU swap, bucket-width-for-count trade, pre-emptive load-balancing, and the repeated finding that gains overlap rather than compound.
- **Open questions**: cascade-depth saturation, whether any seams stack, and the overdue second-trace validation.
- **Per-policy notes**: grouped by recency/promote-on-hit, LFU, and the stash tier.


---

- finished: 2026-06-05T19:37:51+00:00
- duration: 25.74s
- status: ok
- input_tokens: 3
- output_tokens: 1651
- cache_read_tokens: 39843
- cache_write_tokens: 9428
- cost_usd: 0.077287
