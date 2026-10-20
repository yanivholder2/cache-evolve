# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:01:53+00:00

## timeline

### 20:02:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Capacity/ratio changes beat rule-level tweaks: shrinking the small queue won in cycles 2 and 3, but gains shrank each step and are now near-saturated.\n- Ghost-side information and strictness helped: ghosting main victims (cycle 1) and a shorter ghost (cycles 3-4) won, but only marginally. The ghost-length and small-ratio gains are not additive.\n- Each cycle's winner was a single small change built on the prior winner, and every gain was small.\n\n## Recurring failures\n\n- Loosening admission hurt in every cycle it was tried: a looser promotion gate, freq=1 on ghost hits, and a longer ghost. Weakly-reused keys pollute main.\n- Faster ageing or decay of main entries lost every time (freq halving, lower freq cap). This workload rewards keeping reuse history.\n- Filtering ghost hits was clearly worse (cycle 4). A ghost hit is strong reuse evidence, so keep that path permissive.\n- A stricter promotion gate was a tie, so it is either non-binding or a no-op diff.\n- Rule-level single-branch tweaks rarely pay off. Admission sizing is now saturated.\n\n## Open questions\n\n- Are the small gains real or noise? All evidence comes from one trace (LIRS) at one cache size. Test a second trace and cache size.\n- Does the very small probation window hurt scan-heavy or other traces?\n- Is S3-FIFO at a local optimum here? Untried structural levers: main-eviction policy, reinsertion rules, and loosening the path for ghost hits only.\n- Did the stricter-gate diff change behaviour at all? Verify that future diffs alter behaviour.\n- Metadata-size and per-op-time have been missing from the fitness vectors for four cycles. Can they be restored so cost can be judged?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far): the current base is main-victim ghosting, a small queue much smaller than the original, and a shortened ghost. Strict filtering plus permissive ghost-hit re-admission is the pattern that works.\n- Stop tuning the small ratio, ghost length, promotion threshold, and freq decay or cap. Move to structural changes and robustness checks.\n"
}
```

### 20:02:07 chat

I wrote `cross-cycle-insights.md` to the cycle 0004 cross-cycle output directory, with the four fixed sections in order. It's regenerated from the four-cycle ledger digest alone; I didn't read the previous file or any code.

- **Recurring wins:** Capacity and ratio changes beat rule-level tweaks, but the gains are small and nearly used up. Ghost-side changes also helped, though the ghost-length and small-queue gains don't add together.
- **Recurring failures:**
  - Anything that loosens admission lost every time it was tried.
  - Faster ageing of main entries lost every time.
  - Filtering ghost hits was clearly worse.
  - A stricter promotion gate tied its parent, so it may have changed nothing.
- **Open questions:**
  - Do the gains hold on a second trace and cache size?
  - Is S3-FIFO already at a local optimum, and which structural levers remain?
  - Can the metadata-size and per-op-time measurements be restored?
- **Per-policy notes:** S3-FIFO is the only policy so far. What works is strict filtering plus a permissive ghost-hit path. The note says to stop tuning ratios, thresholds and decay, and move to structural changes and robustness checks.

I kept the numbers qualitative, as the constraints require.


---

- finished: 2026-10-20T20:02:08+00:00
- duration: 14.74s
- status: ok
- input_tokens: 4
- output_tokens: 1330
- cache_read_tokens: 36381
- cache_write_tokens: 7819
- cost_usd: 0.048632
