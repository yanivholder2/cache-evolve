# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007

## Winners

- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle ran the three threads cycle 0003 left open against the 90/10
SLRU winner: push the protected ratio to 95/5 (H01), layer a
weak-protection ghost on the incumbent (H02), and add a W-TinyLFU
admission window for scan resistance (H03). Only the ratio push won —
v001 (95/5) landed in the Winners, but by the thinnest margin yet. The
ghost was barely perceptible and the admission window actively
regressed.

## Patterns

- The protected-residency lever still pays at 95/5, but its return is
  now nearly spent: shrinking probationary to ~100 slots produced a far
  smaller gain than each prior enlargement. The binding constraint has
  shifted from "appetite for protected tenure" to the probationary floor
  needed to keep draining one-hit-wonders.
- Reuse-distance memory (the ghost) is directionally right but
  structurally redundant against a maximally-protected tier: with 90% of
  capacity already protected, genuinely hot keys are retained anyway, so
  the warm-but-evicted band left for the ghost to rescue is vanishingly
  thin — and it costs a ghost structure plus an O(n) scan per insert.
- A non-recency admission signal is not just inert here but harmful: the
  ~32-slot window is far smaller than this trace's reuse distances, so it
  evicts genuinely reused keys before their second touch. Scan resistance
  assumes one-hit-wonders dominate; on this LIRS trace they do not.

## Open questions

- 95/5 still pays but barely; is there any room left above it before the
  probationary segment can no longer host the eviction path, or is this
  effectively the peak?
- Both off-axis levers (ghost, admission window) have now been tested in
  isolation *and* layered on the incumbent and failed both times — is
  there any orthogonal signal in this trace left to exploit, or is the
  recency/promotion axis fully consumed?

## Continuation

Treat the protected-ratio lever as essentially exhausted: 95/5 is the
last marginal win and probably at or adjacent to the ceiling — do not
spend a future cycle on 97/3-style probes expecting real gains. Ghost
re-admission and front admission windows are now confirmed dead on this
trace (redundant or harmful); drop both permanently. The cohort has
consumed nearly all recency/promotion structure; the next cycle should
either accept the incumbent as a converged optimum or look for a
genuinely different workload signal rather than re-tuning SLRU geometry.


### Cycle 0005

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle followed up cycle 0005's two live threads: tune the LIRS
rebuild by transplanting SLRU lessons onto it (H01 widen the resident-HIR
window, H02 route re-admitted ghost hits to the weak tier), and close the
last SLRU geometry question by pushing the protected ratio to 97/3 (H03).
All three regressed. v003 (the 97/3 probe) won only as the
least-regressive change, and it did so by confirming a negative.

## Patterns

- The SLRU geometry axis is now formally closed at 95/5. Pushing to 97/3
  shrank the probationary reuse window below the floor needed to catch
  second touches, and the marginal protected slots were worth less than
  the drain capacity they consumed — the binding constraint is the
  probationary floor's *size*, exactly as prior cycles diagnosed.
- SLRU tuning lessons do not transplant onto LIRS. Both LIRS levers
  failed for structural reasons: the HIR window and LIR budget are
  zero-sum, so widening the reuse-detection queue directly evicts the
  low-IRR blocks LIRS was protecting; and a non-resident-HIR reuse is
  itself the canonical LIR-qualifying signal, so demoting it to the weak
  tier (the SLRU ghost-placement win) withholds protection LIRS correctly
  grants. LIRS's own S-membership and stack pruning already do the
  filtering SLRU bolted on after the fact.
- The cross-policy transfer hypothesis is disconfirmed: lessons learned
  inside one skeleton are not portable as parameter edits to another with
  a different invariant structure.

## Open questions

- Cycle 0005's central question — is v002's reported LIRS lift real and
  does it beat the 95/5 SLRU incumbent? — remains unresolved. This cycle
  only tried to *tune* LIRS (and failed both tunings); it never cleanly
  re-ran the untuned LIRS parent against the incumbent to confirm or
  refute the lift.
- With both LIRS tuning levers regressing from the parent config, is the
  LIRS parent at its own local optimum, or is there an untested lever
  (LIR/HIR warm-up, non-resident bound) that actually moves it?

## Continuation

Treat the SLRU line as fully converged at 95/5 — do not probe geometry
again. The one unfinished piece of business is the LIRS verification
cycle 0005 called for and this cycle skipped: next cycle should cleanly
re-run the untuned LIRS parent, confirm whether its hit-rate lift is real
and beats the 95/5 SLRU incumbent on its home-turf trace, and verify the
observable contract. Do not attempt further LIRS tuning by transplanting
SLRU parameters — that transfer is disconfirmed; tune LIRS only against
its own measured behavior. If the untuned LIRS lift does not survive a
clean re-run, declare the cohort converged on the 95/5 SLRU optimum.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
