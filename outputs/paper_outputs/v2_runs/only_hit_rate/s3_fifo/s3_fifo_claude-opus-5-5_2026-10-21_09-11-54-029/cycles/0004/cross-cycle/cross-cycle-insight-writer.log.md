# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:29:48+00:00

## timeline

### 09:30:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Giving main more capacity has paid off for two cycles running (cycles 1–2). The small queue went from baseline to 10% to 5%, and the gain each time came from a bigger main queue, not a bigger ghost. The curve is now nearly flat.\n- On workload-a, protecting keys with proven reuse is the one lesson that has held every cycle. Changes that guard main-queue residents (smaller small queue, higher frequency cap up to 7) helped. Changes that made it easier to enter main did not.\n- It works to separate representation from policy. Behaviour-neutral indexed ports (cycles 1, 3, 4) matched their parents' hit-rate exactly each time. That means structure and policy changes can be stacked safely.\n- Changing one thing per variant makes results easy to attribute. The one variant that changed two things together (cycle 1's v002) couldn't be explained.\n\n## Recurring failures\n\n- Ghost re-admission changes have lost hit-rate in every form tried:\n  - extra credit (cycle 2)\n  - tail-of-main insertion (cycle 3)\n  - small-queue probation (cycle 4)\n  - a larger or decoupled ghost (cycle 2)\n\n  A bare ghost hit is enough evidence of reuse, and front-of-main at freq 0 is the settled point. Stop exploring this axis.\n- Earlier graduation from the small queue (cycle 1) hurt. Easy entry pushes proven keys out of main.\n- Raising the frequency cap above 7 stopped paying off (cycle 4 ended in a tie). Eviction victims are mostly freq-0 re-entries, so extra credit rarely changes which key is evicted. The higher cap only makes sweeps longer.\n- Per-op-time and metadata-size have not been measured in any of the four cycles, even though every Continuation asked for them. Every ghost-growing or cap-raising Winner adds a scan or sweep cost that nobody has measured. That leaves the Winner choices only partly justified.\n\n## Open questions\n\n- How much per-op-time does the indexed port save, and how much metadata-size does it add (duplicated keys, stale ghost deque, compaction spikes)? This is the main blocker. Measure it first.\n- Does cap 15 on the linear structure cost enough per-op-time to make it a net loss compared with cap 7?\n- Is there hit-rate headroom left on workload-a now that partition size, cap and re-admission are flat or settled? Credit aging or decay in main is the main retention lever not yet tested.\n- Could decoupling the graduation threshold from the cap help, with the threshold changed alone and no frequency reset?\n- Do these gains hold on other traces and cache sizes? Every result so far comes from a single workload.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far): the effective baseline is a 5% small queue, cap 7 and front-of-main freq-0 ghost re-admission, running on the indexed slab/index/lazy-ghost structure.\n- S3-FIFO partition: improved from baseline to 10% to 5%, with diminishing returns. Going below 5% is probably not worth it.\n- S3-FIFO ghost: it does most of the filtering at small partition sizes. Keep it coupled to main and non-generous.\n- S3-FIFO main retention: the cap axis paid off up to 7 and is flat beyond that. Future work should change how credit is spent or aged, not how much credit there is.\n"
}
```

### 09:30:06 thinking



### 09:30:06 chat

I've written `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder, using the four required sections in order and built only from the ledger.

- **Recurring wins:** Shrinking the small queue (baseline to 10% to 5%) helped because main got bigger, not because the ghost did. Keeping keys that have already shown reuse is the one lesson that held every cycle. Behaviour-neutral ports onto the indexed structure matched their parents' hit-rate each time, so policy and structure changes can be stacked.
- **Recurring failures:**
  - Every form of making ghost re-admission more generous or stricter lost hit-rate: extra credit, tail insertion, small-queue probation, and a larger ghost.
  - Earlier graduation from the small queue hurt.
  - Raising the frequency cap above 7 stopped paying off.
  - Per-op-time and metadata-size have not been measured in any of the four cycles, so no Winner has been judged on cost.
- **Open questions:**
  - What the indexed port actually saves in per-op-time and adds in metadata-size. This is the main blocker.
  - Whether cap 15 is a net loss once its cost is measured.
  - Whether credit aging or decay in main adds any hit-rate.
  - Whether the graduation threshold should be decoupled from the cap.
  - Whether the gains hold on other traces and cache sizes.
- **Per-policy notes:** Only S3-FIFO has been explored. The effective baseline is a 5% small queue, cap 7, and ghost hits re-entering at the front of main with zero frequency, running on the indexed structure.


---

- finished: 2026-10-21T09:30:06+00:00
- duration: 18.66s
- status: ok
- input_tokens: 4
- output_tokens: 1837
- cache_read_tokens: 36297
- cache_write_tokens: 8916
- cost_usd: 0.115343
