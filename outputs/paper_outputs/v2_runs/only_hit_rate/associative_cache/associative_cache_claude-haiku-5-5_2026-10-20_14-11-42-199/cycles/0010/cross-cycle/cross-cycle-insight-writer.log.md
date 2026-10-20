# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:52:08+00:00

## timeline

### 14:52:17 thinking



### 14:52:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Shared-frequency admission or victim comparison, which keeps one-shot keys from displacing keys with proven reuse, has produced the only consistent hit-rate gains (Cycles 0004, 0005, 0010). Magnitudes are small and single-trace.\n- Scan-resistant probationary-first eviction is the other repeated gain source (Cycle 0003), and it is the same family as the admission gate.\n- Recurrence-gated admission through a one-capacity ghost list showed a gain once (Cycle 0007). It has not been retested since, so it is a lead, not a win.\n\n## Recurring failures\n\n- Changing where newly admitted keys sit in the eviction order has lost hit-rate repeatedly: front of probation (Cycle 0006), victim end (Cycles 0007, 0008), and eviction-next stacked on the gate (Cycle 0009). Only one eviction-next result (Cycle 0005) was positive, and its placement semantics are not confirmed to match.\n- Aging by halving or pruning counters has lost hit-rate in every form tried: zero-pruning (Cycle 0009), plain floor-halving (Cycle 0010), and singleton pruning (Cycle 0008). Small counter magnitudes erase the one-hit versus no-hit distinction the gate depends on.\n- Protected-tier and second-hit variants have not held up (Cycles 0004 to 0006). Shrinking the protected region changed nothing, and removing forced demotion did not recover the loss.\n- Loosening the admission comparison, whether by ties, by bypassing the counter check, or by stacking a recurrence refusal in front of the gate, lowers hit-rate (Cycles 0007, 0009, 0010).\n- Widening the ghost-list window (Cycle 0008) and bounding the victim scan as a hit-rate lever (Cycles 0003, 0004) gave no gain.\n- Hit-rate-only geometry tweaks (Cycle 0001) produced at most marginal gains with no cost evidence.\n\n## Open questions\n\n- Winner labels still conflict with the recorded evidence in several cycles (0004, 0006 to 0009). Cycle 0010 is the first where the label matches the data. The selection rule should be checked before any Winner is used as a precedent.\n- Cost is unmeasured in every cycle. Metadata size and per-op time are missing for the shared counter, the ghost list, the per-miss probationary scan, and the full halving pass. The cost side of every hypothesis is still unverified.\n- Does the Cycle 0010 victim-by-frequency gain hold on a second trace and cache size, and does its extra per-miss scan cost offset it?\n- Does the Cycle 0007 recurrence gain survive when the counter check is kept on the ghost path, as Cycle 0010 suggests?\n- Is the protected-tier failure caused by the second-hit signal or by the demotion rule?\n- Is the eviction-next gain of Cycle 0005 a different mechanism from the victim-end and front-placement losses?\n\n## Per-policy notes\n\n- Shared-frequency admission / victim selection (LFU-like): the most consistent positive direction. Keep the comparison strict, and test extra cost before accepting it.\n- Aging and counter halving (AgedLfu-style): negative in every variant so far. Do not retry removal or floor-based halving without keeping previously seen counts at or above 1.\n- Two-region / protected tier (SLRU-like): repeated failures. Capacity is not the lever; the second-hit or demotion rule is the open suspect.\n- Ghost-list / recurrence gating: one positive result (Cycle 0007), with mixed results when stacked or overridden. Keep the one-capacity window and the counter check as the base for any retest.\n- Insert position (MRU, victim end, eviction-next, front of probation): placement changes that make new keys eviction-eligible sooner have mostly lost. State the insert position explicitly in every hypothesis.\n- Bucket geometry and fixed-width buckets (Cycle 0001): no cost-side evidence; treat as unexplored until paired with per-op time.\n"
}
```

### 14:52:25 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle output directory, with the four required sections in order. I didn't read any previous version of the file.

The main points:

- **Recurring wins:** Shared-frequency admission or victim comparison is the only direction with repeated hit-rate gains (Cycles 0004, 0005, 0010). Those gains are small and single-trace. The recurrence-gated ghost list (Cycle 0007) is listed as a lead, since it has not been retested.
- **Recurring failures:** Placement changes that make newly admitted keys eviction-eligible sooner have mostly lost. Counter aging lost in every form tried. Protected-tier variants haven't held up. Loosening the admission comparison also lowers hit-rate.
- **Open questions:** The Winner label conflicts with the recorded evidence in several cycles (0004, 0006–0009). Cycle 0010 is the first where it matches. Metadata size and per-op time are unmeasured in every cycle.
- **Per-policy notes:** Grouped by shared-frequency admission, aging/halving, two-region tiers, ghost-list gating, insert position, and bucket geometry.


---

- finished: 2026-10-20T14:52:26+00:00
- duration: 17.37s
- status: ok
- input_tokens: 4
- output_tokens: 3288
- cache_read_tokens: 42219
- cache_write_tokens: 15655
- cost_usd: 0.005198
