# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **Reward proven reuse; keep the entry point cheap and unprotected.** Every cycle (0001–0009) supports this. Re-admit a ghost hit at front-of-main with no credit (freq 0). Let graduates carry only the credit they earned in small.
- **A bigger main queue (a smaller small queue) helped up to about 5%.** The 20%→10%→5% sweep gave gains (0001–0002). The gains came from main capacity, not from ghost size, and the curve was flattening by 5%.
- **Main-queue eviction order was the last real hit-rate lever.** A SIEVE hand that leaves survivors in place (0007), with credit cleared on each pass (0008), gave the only clear gains after the knobs looked saturated. One visited lap, plus credit carried over from small, now looks locally optimal (0009).
- **Widen the ghost's evidence a little without making re-admission more generous.** Ghosting main victims inside the shared, capped budget gave a small win (0005).
- **Representation changes that keep the policy identical compose reliably.** The indexed slab, fingerprint ring, 32-bit fingerprints and identity or cheap hashing all matched their parents exactly or within noise against full-state references (0003–0009). Each later became the baseline.

## Recurring failures

- **Making re-admission more generous or stricter loses either way.** Extra credit for re-admitted keys (0002), tail insertion (0003) and small-queue probation (0004) all lost. One bare ghost hit is enough evidence. The small queue's window is too short to re-filter what the ghost already found.
- **Adding ghost capacity loses.** A larger or decoupled ghost (0002) and a separate budget for main victims (0006) both lost. Every ghost hit bypasses the small filter and forces a main eviction, so the capped shared budget protects main residents.
- **Changes to graduation from small are flat or negative.** A lower threshold, with or without a frequency reset (0001, 0005), and a reset at promotion (0009) did not help. The ghost already recovers keys hit once in small, and the credit carried from small is real evidence.
- **Extra retention under SIEVE loses.** Behind-hand newcomer insertion (0008) and a second lap for freq ≥ 2 (0009) both lost. Under FIFO reinsertion, cap and decay also stopped helping at cap 7 (0004–0005).
- **Adaptive splits driven by ghost-hit source lost (0006).** The signal is biased toward small-queue rejects and arrives too late to help.
- **Cost evaluation is a standing gap.** Per-op-time and metadata-size have gone unmeasured for nine cycles, and every analysis has tested only workload-a at one size. Every cost-side "win" so far was decided by hit-rate parity alone.

## Open questions

- Do the cost-side changes actually reduce per-op-time and metadata-size? This covers the indexed slab, fingerprint ring, 32-bit fingerprints, cheap hashing and SIEVE's pointer-chasing sweep. It is the top priority and is overdue.
- Do the accumulated choices (5% split, SIEVE with clear-on-pass, ghosted main victims) hold on other traces and cache sizes, or are they fitted to workload-a?
- Under SIEVE, should ghost re-admissions get even less protection, for example insertion nearer the hand? This is optional, low-priority policy work.
- Was the 5% split, tuned under FIFO reinsertion, still the right point under SIEVE? This is untested, but low priority.
- Would a cleaner two-lap test that counts only main hits behave differently? Low priority, because both neighbouring points regressed.

## Per-policy notes

- **FIFO-reinsertion main (cycles 0001–0006):** Results were a 5% split, cap 7 with a linear decrement, and the ghost capped and shared. Decay and caps above 7 were flat or negative. These retention verdicts do not carry over to SIEVE.
- **SIEVE main (cycles 0007–0009):** Keeping survivors in place beat FIFO reinsertion, and clear-on-pass beat multi-lap credit. Both more retention (a second lap, behind-hand insertion) and less carried credit (a reset at graduation) lost. The current baseline is clear-on-pass SIEVE on 32-bit identity-hashed fingerprints.
- **Ghost (all cycles):** Re-admit at front-of-main with freq 0, use one shared capped budget, and include main victims. The representation has moved from cloned keys to a fingerprint ring to 32-bit identity-hashed fingerprints, all with the policy unchanged.
- **Small queue / graduation:** The current setup is a 5% partition with graduation and main retention sharing one counter. Separating them, lowering the threshold or resetting credit all gave nothing. Treat this area as settled on workload-a.
