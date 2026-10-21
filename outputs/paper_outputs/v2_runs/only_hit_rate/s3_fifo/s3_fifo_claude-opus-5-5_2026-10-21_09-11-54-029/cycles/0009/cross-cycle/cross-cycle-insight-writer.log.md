# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T14:43:57+00:00

## timeline

### 14:44:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- **Reward proven reuse; keep the entry point cheap and unprotected.** Every cycle (0001–0009) supports this. Re-admit a ghost hit at front-of-main with no credit (freq 0). Let graduates carry only the credit they earned in small.\n- **A bigger main queue (a smaller small queue) helped up to about 5%.** The 20%→10%→5% sweep gave gains (0001–0002). The gains came from main capacity, not from ghost size, and the curve was flattening by 5%.\n- **Main-queue eviction order was the last real hit-rate lever.** A SIEVE hand that leaves survivors in place (0007), with credit cleared on each pass (0008), gave the only clear gains after the knobs looked saturated. One visited lap, plus credit carried over from small, now looks locally optimal (0009).\n- **Widen the ghost's evidence a little without making re-admission more generous.** Ghosting main victims inside the shared, capped budget gave a small win (0005).\n- **Representation changes that keep the policy identical compose reliably.** The indexed slab, fingerprint ring, 32-bit fingerprints and identity or cheap hashing all matched their parents exactly or within noise against full-state references (0003–0009). Each later became the baseline.\n\n## Recurring failures\n\n- **Making re-admission more generous or stricter loses either way.** Extra credit for re-admitted keys (0002), tail insertion (0003) and small-queue probation (0004) all lost. One bare ghost hit is enough evidence. The small queue's window is too short to re-filter what the ghost already found.\n- **Adding ghost capacity loses.** A larger or decoupled ghost (0002) and a separate budget for main victims (0006) both lost. Every ghost hit bypasses the small filter and forces a main eviction, so the capped shared budget protects main residents.\n- **Changes to graduation from small are flat or negative.** A lower threshold, with or without a frequency reset (0001, 0005), and a reset at promotion (0009) did not help. The ghost already recovers keys hit once in small, and the credit carried from small is real evidence.\n- **Extra retention under SIEVE loses.** Behind-hand newcomer insertion (0008) and a second lap for freq ≥ 2 (0009) both lost. Under FIFO reinsertion, cap and decay also stopped helping at cap 7 (0004–0005).\n- **Adaptive splits driven by ghost-hit source lost (0006).** The signal is biased toward small-queue rejects and arrives too late to help.\n- **Cost evaluation is a standing gap.** Per-op-time and metadata-size have gone unmeasured for nine cycles, and every analysis has tested only workload-a at one size. Every cost-side \"win\" so far was decided by hit-rate parity alone.\n\n## Open questions\n\n- Do the cost-side changes actually reduce per-op-time and metadata-size? This covers the indexed slab, fingerprint ring, 32-bit fingerprints, cheap hashing and SIEVE's pointer-chasing sweep. It is the top priority and is overdue.\n- Do the accumulated choices (5% split, SIEVE with clear-on-pass, ghosted main victims) hold on other traces and cache sizes, or are they fitted to workload-a?\n- Under SIEVE, should ghost re-admissions get even less protection, for example insertion nearer the hand? This is optional, low-priority policy work.\n- Was the 5% split, tuned under FIFO reinsertion, still the right point under SIEVE? This is untested, but low priority.\n- Would a cleaner two-lap test that counts only main hits behave differently? Low priority, because both neighbouring points regressed.\n\n## Per-policy notes\n\n- **FIFO-reinsertion main (cycles 0001–0006):** Results were a 5% split, cap 7 with a linear decrement, and the ghost capped and shared. Decay and caps above 7 were flat or negative. These retention verdicts do not carry over to SIEVE.\n- **SIEVE main (cycles 0007–0009):** Keeping survivors in place beat FIFO reinsertion, and clear-on-pass beat multi-lap credit. Both more retention (a second lap, behind-hand insertion) and less carried credit (a reset at graduation) lost. The current baseline is clear-on-pass SIEVE on 32-bit identity-hashed fingerprints.\n- **Ghost (all cycles):** Re-admit at front-of-main with freq 0, use one shared capped budget, and include main victims. The representation has moved from cloned keys to a fingerprint ring to 32-bit identity-hashed fingerprints, all with the policy unchanged.\n- **Small queue / graduation:** The current setup is a 5% partition with graduation and main retention sharing one counter. Separating them, lowering the threshold or resetting credit all gave nothing. Treat this area as settled on workload-a.\n"
}
```

### 14:44:21 chat

I wrote `cross-cycle-insights.md` in the cycle 0009 cross-cycle directory. It has the four required sections in the required order and is built only from the ledger.

The two biggest gaps across all nine cycles:
- **Nothing on cost has been measured.** Per-op-time and metadata-size were never reported. Every cost-side change won only because its hit-rate stayed the same as its parent's.
- **Only one workload was tested.** Every result comes from workload-a at one cache size, so it's unknown whether any of the gains carry over to other traces or sizes.

What the file records:
- **What kept working:** the policy rewards keys that have already shown reuse. Re-admitting a ghost hit at the front of main with no credit was the best setting. A larger main queue helped up to a 5% small queue. Changing how main evicts gave the last real gains: a SIEVE-style hand from cycle 0007, then clearing credit on each pass from cycle 0008.
- **What kept failing:** every change to re-admission, in both directions. Larger ghost budgets, changes to graduation from the small queue, and extra retention under SIEVE all lost too, as did the adaptive split.
- **Open questions:** first, measuring per-op-time and metadata-size and testing a second trace or cache size. After that, a few low-priority policy probes.
- **Notes by policy:** results under the old main-queue eviction (cycles 0001–0006) are kept separate from SIEVE results (cycles 0007–0009). The earlier settings for retention caps and credit decay don't carry over to SIEVE.


---

- finished: 2026-10-21T14:44:22+00:00
- duration: 24.32s
- status: ok
- input_tokens: 4
- output_tokens: 2335
- cache_read_tokens: 41124
- cache_write_tokens: 14272
- cost_usd: 0.169117
