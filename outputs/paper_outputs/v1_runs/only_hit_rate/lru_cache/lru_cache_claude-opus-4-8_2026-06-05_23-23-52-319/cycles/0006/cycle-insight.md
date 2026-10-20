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
