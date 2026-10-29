# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007's v001, and each changed one lever behind the 32_768 size gate. v001 gave the doorkeeper four times the bits at small size. v002 cut the protected share to 75% at large size. v003 damped the climber's step on each flip at large size. v002 is the Winner and the second strict gain in a row. Large-size web_0 improved, large-size hm_0 lost a little, and small size was unchanged.

## Patterns

- Size gates held every time. Each variant left the cells on the other side of its gate exactly unchanged. Gating levers by size is now a reliable tool. The open question is which lever to gate, not whether gating works.
- Large-size gains have become a trade between web_0 and hm_0. The step gate, the split gate and the damping rule all moved hit-rate along the same line, gaining on one large trace and losing on the other. Only the split gate came out net positive. Fixed rules that ignore the workload seem close to their limit at large size.
- The doorkeeper hypothesis for small-size web_0 is now tested and rejected. A saturated doorkeeper acts like a roughly uniform +1 bonus on first touches. Making it more accurate removed that bonus unevenly, hurt moderate-reuse keys on small-size hm_0 and did nothing for web_0. Small-size web_0 has now resisted climber, split, admission-loosening and doorkeeper changes.
- Small-size hm_0 is still the fragile cell. Any change that alters how moderate-reuse keys are scored against victims hurts it first.
- For the eighth cycle, only hit-rate was measured. `win_cap`, `metadata-size` and `per-op-time` are still not reported.

## Open questions

- Can a runtime signal tell scan-heavy traffic from medium-reuse traffic at large size? Candidates are the hit-rate response after a flip, how scan-heavy the window traffic is, or a climber or duel that also adjusts the protected split.
- What limits small-size web_0? The candidates left are the admission tie-break, the sketch's `reset_at` period, and the window/main split. None has been tested directly at small size only.

## Continuation

Use cycle 0008 v002 as the new parent, with the 75% large-size split. Keep the doorkeeper, the 80% small-size split and the climber settings as they are. Next, try a lever that adapts to the workload at large size, for example letting the protected split adapt at runtime. Separately, test the tie-break or `reset_at` at small size only. Drop doorkeeper resizing, flip damping and further static large-size gates on the step or split. Reading `win_cap` and reporting `metadata-size` and `per-op-time` is now eight cycles overdue.
