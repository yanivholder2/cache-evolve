# Cycle 0005 Insight

## Cohort outcome

After three cycles of exhausting SLRU geometry, this cycle finally
reached off the recency/promotion axis: an LFU protected-tier victim
selector (H01), a from-scratch canonical LIRS rebuild (H02), and a
front-vs-back demotion-placement tweak (H03). The LFU selector regressed
and the placement tweak was inert; v003 (demotion placement) landed in
the Winners as the neutral, least-regressive change. Notably, the LIRS
rebuild (v002) reported a hit-rate lift on its home-turf trace yet did
**not** win — the standout thread of the cycle.

## Patterns

- The recency/promotion axis is confirmed fully consumed. Two more
  in-skeleton levers (frequency at the demotion point, demotion
  placement within probationary) joined the pile of inert-or-harmful
  SLRU tweaks: at a near-full 95% protected tier, freq values are tied so
  LFU just mistracks recency and demotes recently-hot keys, and the
  binding constraint remains the probationary floor's *size*, not the
  order within it.
- Frequency is not the untapped axis cycle 0004 speculated about. Applied
  at the retention point it underperforms plain LRU here, mirroring the
  earlier admission-window failure — non-recency signals keep losing on
  this trace inside the SLRU frame.
- The one genuinely different mechanism — reorganising eviction priority
  around measured inter-reference recency (LIRS), rather than bolting
  re-admission onto SLRU — is the first off-axis probe to report a real
  gain. This is the "different workload signal" prior cycles kept asking
  for, and the trace family is its home turf.

## Open questions

- Why did v002's reported LIRS gain not land in the Winners? Was the lift
  within noise, or did the rebuild trip a contract/invariant check that
  disqualified it despite higher hit-rate? This must be resolved before
  trusting it.
- If the LIRS gain is real, does it dominate the 95/5 SLRU incumbent
  outright, or only on this specific LIRS-derived trace?

## Continuation

Stop tuning SLRU geometry and stop testing non-recency signals *inside*
the SLRU frame — both are exhausted. The live thread is the canonical
LIRS rebuild (v002): next cycle should re-run it cleanly, confirm whether
its reported hit-rate lift is real and beats the 95/5 incumbent, and
verify it preserves the observable contract. If it holds, make LIRS the
new parent and tune its LIR/HIR split; if it was noise or disqualified,
treat the cohort as converged on the 95/5 SLRU optimum. Do not revisit
LFU victim selection or demotion placement.
