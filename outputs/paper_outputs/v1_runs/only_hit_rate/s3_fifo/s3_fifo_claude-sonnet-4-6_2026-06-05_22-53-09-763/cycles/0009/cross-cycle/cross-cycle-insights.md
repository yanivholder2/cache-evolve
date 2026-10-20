## Recurring wins

- **Small-queue shrinkage consistently improves hit rate** by transferring capacity to main and raising resident quality. The benefit has held monotonically from 25% → 10% → ~8% → ~7% → ~5% across cycles 0001, 0002, 0008, and 0009; the true floor is still unlocated empirically.
- **Ghost queue precision at the main_size/2 cap** is the workload's optimal signal window — confirmed from both sides: larger windows admit noise from non-recurring distant accesses (cycle 0001), smaller windows discard genuine re-use signal (cycle 0003).
- **MAX_FREQ ceiling increases yield real but rapidly diminishing gains.** Each upward step (3→4, 4→5) produced a positive result with shrinking magnitude; the direction is confirmed valid but is approaching exhaustion.
- **Unconditional move-to-front on all main-queue hits** is a confirmed small positive (cycles 0005, 0008, 0009), addressing positional drift as a secondary inefficiency orthogonal to admission quality.
- **Stacking orthogonal wins compounds additively.** The cumulative best configuration is built from independent mechanisms (small-queue size, ghost cap, frequency ceiling, MTF) that each address a distinct failure mode without cancelling one another.

## Recurring failures

- **Relaxing the promotion threshold (freq > 0):** tried solo and in combination across cycles 0003 and 0004; always neutral or harmful. The narrow small queue is insufficient to compensate for the extra cold traffic admitted to main.
- **Extending the ghost window above main_size/2:** hurts in cycle 0001 and remains banned; the workload's re-use distances do not justify a wider history.
- **Ghost re-admission head-start credits (seeding at freq=1):** displaces legitimate incumbents without recovering the expected promotion benefit (cycle 0004); the ghost signal confirms re-use potential but not priority over established residents.
- **Adaptive threshold control on a phase-stable workload:** the control signal rarely fires, and when it does it acts on noise rather than a real phase transition — the mechanism regressed in cycle 0007 for exactly this reason.
- **Positional manipulation within main** (midpoint insertion, saturation-triggered head-repositioning): the conditions under which a position rescue would help almost never arise on a Zipfian workload; confirmed inert in cycle 0006.
- **Tightening the second-chance threshold post-eager-promotion:** eager promotion deposits freq==2 items into main; an absolute frequency gate then ejects freshly-promoted hot items, reversing the intended filter effect (cycle 0008).
- **Extending the freq==1 eviction recycle window under eager promotion:** items at the eviction tail carrying freq==1 after promotion are genuinely cold on this workload; extra passes delay fresher admissions without improving their own hit probability (cycle 0009).

## Open questions

- **True small-queue floor:** the monotone sizing benefit has never turned negative; does it break sharply at some threshold (filter becomes too coarse) or taper gradually? Probing below ~5% (e.g., capacity/28 or capacity/40) is the highest-priority empirical question.
- **Small-queue + MTF interaction at the current base:** both mechanisms are confirmed positives targeting orthogonal failure modes; their combined effect at ~5% small-queue size has not been directly measured.
- **Frequency-relative second-chance threshold:** a gate formulated relative to the post-promotion frequency distribution might capture the original stale-squatter intent without penalising eager-promoted items — not yet explored.
- **Adaptive mechanisms on phase-shifting workloads:** cycle 0007's adaptive variant failed on phase-stable workload-a but its design may be sound; a scan-heavy or hot-set-transitioning trace would test whether the control surface is useful anywhere.
- **Residual gap vs. alternative algorithms:** all S3-FIFO knobs are near-exhausted; whether TinyLFU or SIEVE would outperform the current best configuration on workload-a is an open architectural question.

## Per-policy notes

- **S3-FIFO — small queue:** functions as a cold-admission filter; optimal size shrinks monotonically on this Zipfian workload and the floor has not yet been reached after nine cycles of tuning.
- **S3-FIFO — ghost queue:** optimal cap is main_size/2, confirmed as both a ceiling and a floor on workload-a; ghost signal quality depends on precision (recency-bounded eviction history), not coverage (total history breadth).
- **S3-FIFO — main queue / second-chance:** move-to-front is the primary recency-sorting mechanism — replacing it with CLOCK-style in-place scanning caused the largest single regression observed (cycle 0005). Unconditional MTF on all hits is a net positive; the second-chance frequency threshold (freq > 1) is well-matched to the post-eager-promotion entry distribution and should not be tightened with absolute values.
- **S3-FIFO — eager promotion interaction:** introducing eager promotion (cycle 0007) silently changed the frequency distribution of main-queue arrivals. Threshold changes that were safe or beneficial before eager promotion may regress afterward; any future threshold experiment must account for the freq==2 entry distribution created by that mechanism.
- **S3-FIFO — MAX_FREQ:** confirmed valid range is 3–5 on this workload; 3 is the functional lower bound (hot items use the third pass), 5 produces only trace improvement over 4. Further ceiling increases are low-yield.
