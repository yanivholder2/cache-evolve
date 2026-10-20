# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:28:41+00:00

## timeline

### 14:28:50 thinking



### 14:28:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- No variant has shown a demonstrated hit-rate gain over the parent. \"Winner\" has consistently meant \"did not regress.\"\n- Behavior-preserving cost refactors (resident-key probes, ghost-lookup restructuring, membership bit packing, early return on resident hits) have repeatedly stayed hit-rate neutral, as intended. They are the only lever family that reliably passes the gate, but none has been shown to reduce cost.\n- The two-hit small-queue promotion bar is worth keeping. Relaxing it lost hits, and the bar has not been beaten by raising it.\n\n## Recurring failures\n\n- Main-queue frequency tuning is inert. Raising the frequency cap and halving the decay step both came back flat, so frequency-axis tweaks have shown no signal.\n- Ghost-path changes have lost hits in both directions. Widening the ghost window lost hits, and routing ghost-hit re-admissions through the small queue also lost hits. Ghost admission and window size are not yet a source of gains.\n- Adaptive small-queue sizing driven by ghost hits lost hits. The ghost signal appears too noisy to steer the split. Fixed-size small-queue tweaks were also neutral.\n- Main-queue retention changes lost hits. Keeping warm residents longer by changing reinsert position displaced value rather than adding it.\n- Promotion-bar tweaks are saturated. Relaxing lost hits; raising was neutral.\n- Cost is never measured. No cycle has reported metadata-size or per-op-time, so no cost-side candidate can be ranked against the baseline.\n- Overlay verification keeps failing. Several variants had edits that were absent, not confirmed, or not inspected, so their results cannot be tied to the intended change.\n\n## Open questions\n\n- Do the cost-side candidates (resident-key probe variants, ghost-lookup refactor, membership bit packing, early return) reduce per-op-time or metadata-size? A rerun reporting both is needed before any is called a win.\n- Did the intended edits land in the built code for the variants whose overlays were not confirmed or not inspected? Until verified, flat or negative results say little about the intended changes.\n- Was the layout variant's hit-rate drop real, or an artifact of the mismatched overlay? It needs a clean-build rerun.\n- Is the insensitivity of frequency, sizing, and promotion levers a property of this trace, or of the single capacity? Results come from one trace and one capacity only.\n- Would ghost re-admissions behave differently under a different promotion bar, or if ghost hits were admitted by a different path?\n- Does the comparison baseline match the intended parent for each variant? One variant cited another variant as its control, so attribution needs confirmation.\n\n## Per-policy notes\n\n- S3-FIFO, small queue: the promotion bar is the most consistently informative knob. Relaxing it lost hits and raising it was flat. Queue sizing, fixed or adaptive, has not produced gains.\n- S3-FIFO, main queue: frequency cap and decay changes were flat. Reinsert position and retention changes lost hits. The current head reinsert appears to be leaving no obvious value on the table on this trace.\n- S3-FIFO, ghost queue: window widening and small-queue re-routing of ghost hits both lost hits. Any future ghost-size work must also change how ghost hits are admitted, and should be tested on more than one trace or capacity.\n- S3-FIFO, membership and lookup structures: restructuring that preserves eviction order has stayed hit-rate neutral. Its benefit is untested because cost metrics are missing.\n- Evidence base for all of the above: one trace (lirs workload-a) at one capacity (2048).\n"
}
```

### 14:28:58 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output directory, using the four required sections in order. I did not read any previous version of the file.

The main signals across the six cycles:

- **No demonstrated gains.** Every "Winner" has been hit-rate neutral at best. The recurring win is that behavior-preserving cost refactors pass the gate, but none has shown a cost reduction because metadata-size and per-op-time have never been reported.
- **Recurring losses.** Ghost-path changes lost hits in both directions, adaptive small-queue sizing lost hits, and main-queue retention changes lost hits. Frequency-cap and decay tweaks were flat. The two-hit promotion bar appears worth keeping.
- **Verification gaps.** Several variants had overlay edits that were absent, unconfirmed, or not inspected, and one variant cited another as its control. Those results can't yet be attributed to the intended changes.
- **Evidence base.** Everything comes from one trace (lirs workload-a) at one capacity (2048).

The file is 47 lines. I didn't run any other tools or make any commits.


---

- finished: 2026-10-20T14:28:59+00:00
- duration: 17.89s
- status: ok
- input_tokens: 4
- output_tokens: 3356
- cache_read_tokens: 38418
- cache_write_tokens: 11942
- cost_usd: 0.004451
