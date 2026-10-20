## Recurring wins

- Eviction *ordering*, not extra residency, is the durable lever: every
  early gain came from deciding which resident dies first under a fixed
  capacity bound, anchored by a "protect on second access" signal.
- Enlarging the SLRU protected tier paid repeatedly and monotonically
  (80/20 to 90/10 to 95/5), each step landing the winner — until the
  returns thinned to near-zero at the top of that range.
- When an off-axis lever did help, it was placement at the *weak/LRU end*
  of protection (routing ghost re-admissions to protected-LRU) rather
  than straight-to-hot promotion.

## Recurring failures

- Loosening the promotion signal — wider probationary dwell, two-hit
  promotion gates, depth/frequency-stratified protected tiers — always
  regresses: reuse on this trace arrives fast and within a narrow window,
  so anything that delays promotion or steals protected capacity loses
  hits.
- Non-recency admission signals (W-TinyLFU windows, LFU victim selection,
  frequency at the demotion point) are inert-to-harmful: this trace is
  not one-hit-wonder dominated, so scan-resistance machinery evicts
  genuinely reused keys before their second touch.
- Reuse-distance memory (ghosts) is structurally redundant once the
  protected tier is large: hot keys are already retained, leaving only a
  vanishingly thin warm-but-evicted band to rescue.
- Adaptive geometry fails where static geometry succeeds: floating the
  protected/probationary split on reuse feedback drifts to a worse
  partition by eroding the probationary floor.
- Cross-policy transfer is disconfirmed: SLRU tuning lessons applied as
  parameter edits to LIRS regress, because the two skeletons have
  different invariant structures.

## Open questions

- The long-standing unresolved thread (cycles 0005-0007): does an
  *untuned* LIRS parent, cleanly re-run, actually beat the 95/5 SLRU
  incumbent on its home-turf trace, and does it preserve the observable
  contract? Every LIRS attempt so far either only *tuned* (and regressed)
  or reported a lift that never landed in the Winners.
- Is there any remaining structural axis on this trace, or is the 95/5
  SLRU incumbent a true converged optimum? Three consecutive cycles of
  off-axis probes have all regressed.

## Per-policy notes

- SLRU: the workhorse and repeated winner. Binding constraint is the
  *size* of the probationary floor (drain capacity for one-hit-wonders),
  not promotion timing or intra-segment order. Geometry is closed at
  95/5 — both static (97/3 regressed) and adaptive resizing are
  exhausted.
- LIRS: the only genuinely off-axis mechanism to ever report a real gain
  (home-turf trace), but the result is unverified and never won. Its own
  S-membership/stack pruning already does the filtering SLRU bolts on;
  transplanting SLRU knobs (HIR-window widening, weak-tier re-admission,
  wider non-resident horizon) harms it because those signals are zero-sum
  against its LIR budget.
- LRU baselines: dominated from cycle 0001; pure recency is the wrong
  axis and further pure-recency probes were dropped early.
