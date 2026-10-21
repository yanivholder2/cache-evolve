# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:20:14+00:00

## timeline

### 04:20:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing which keys are admitted or how hits are treated beats changing where keys are placed. MRU placement on hit (cycle 1) and the second-sighting admission filter (cycle 3) are the only improvements so far.\n- Gains come from protecting recently useful entries and filtering one-hit-wonders, not from moving more items.\n- Each win built on the prior incumbent (cycle 1 v001, then the admission filter). Branching from the winner is a sound strategy.\n\n## Recurring failures\n\n- Placement-position tweaks beyond the first one (insert path, cold-end demotion, eligibility relaxation) gave equal or marginally better hit-rate. They look exhausted on workload-a.\n- Extra movement (bidirectional swaps, more promotions) causes demotion cascades. Delaying or relocating eviction (wider probe neighbourhood) lets cold entries crowd out hot ones.\n- Several variants matched their parent's hit-rate exactly (cycles 2 and 3). Overlay edits may be dead code, or the workload may be insensitive. This ambiguity wastes cycles.\n- Only hit-rate has been measured in every cycle. Per-op-time and metadata-size are unevaluated, so no cost claims exist.\n- Authors cannot read overlay sources, so all mechanism claims are inferred.\n\n## Open questions\n\n- Do the wins generalize to other workloads and cache sizes? Everything so far is workload-a.\n- What do the admission filter's memory and miss-path hashing cost? Does the gain survive once per-op-time and metadata-size are in the fitness vector?\n- Is the admission threshold tuned well? Would a frequency-based or doorkeeper-style signal do better?\n- Are the edits that reproduced parent hit-rate exactly (especially victim selection) on the live path? Re-test victim selection only once that is confirmed.\n- Does combining the admission filter with MRU placement add anything beyond what each gives alone?\n\n## Per-policy notes\n\n- Dash segment (bucket, probing, stash) with promotion on hit: MRU placement on promotion helps. Eligibility and back-promotion changes do not.\n- Insert and demotion paths: position changes were no-ops on workload-a.\n- Victim selection (furthest-from-MRU scan): neutral, with unverified liveness.\n- Admission (second-sighting filter): the best lever so far. Next step is to tune it and try doorkeeper or frequency variants.\n- Probe structure (wider neighbourhood before the stash): slightly negative. Drop it.\n"
}
```

### 04:20:26 chat

I wrote `cross-cycle-insights.md` to the output dir. It has the four fixed sections, in order, and is a fresh projection of the three-cycle ledger. I didn't read the previous insights file.

- **Recurring wins:** Admission and hit handling beat placement. MRU placement on hit (cycle 1) and the second-sighting admission filter (cycle 3) are the only improvements so far. Each built on the previous incumbent.
- **Recurring failures:**
  - Placement tweaks beyond the first one (insert path, demotion, eligibility) gave equal or marginally better hit-rate.
  - Extra movement and the wider probe neighbourhood hurt or did nothing.
  - Several variants matched their parent's hit-rate exactly, so their edits may be dead code.
  - Only hit-rate has been measured, so there are no cost claims.
- **Open questions:**
  - Do the wins hold beyond workload-a?
  - What does the admission filter cost in memory and per-op time?
  - Is its threshold well tuned?
  - Was the victim-selection edit live?
  - Does combining the admission filter with MRU placement add anything?
- **Per-policy notes:** These are grouped by lever: promotion on hit, insert and demotion, victim selection, admission, and probe structure. Admission is flagged as the lever to pursue next.

I kept the notes qualitative and used no numbers.


---

- finished: 2026-10-21T04:20:27+00:00
- duration: 13.10s
- status: ok
- input_tokens: 4
- output_tokens: 1450
- cache_read_tokens: 35696
- cache_write_tokens: 7221
- cost_usd: 0.047301
