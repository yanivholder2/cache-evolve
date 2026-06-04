# Cycle 0006 Insight

## Cohort outcome

The cohort tested three sharply different bets — fixing the sketch-
marking precondition collapse so the cold-admission gate actually
fires (v001), capping the Hot pool at half capacity with forced
demotion (v002), and the cycle-5-prescribed two-fix S4LRU retry of
asymmetric top-heavy split plus L3 ghost ring (v003). v001 broke the
three-cycle plateau and is the new working champion; v002 and v003
both regressed below the prior champion.

## Patterns

- The plateau finally cracked, and it cracked on the one lever
  cycle 5 explicitly preserved as "exercise it correctly once before
  declaring it dead." A precondition fix — not a new mechanism —
  delivered the first real hit-rate lift in three cycles. The
  cold-admission gate is a genuine lever on this scan-heavy trace,
  vindicating the cycle-3 / cycle-5 calls to keep it alive.
- Two independent structural alternatives to the champion's shape
  lost ground in the same cohort. v002 showed the unbounded Hot pool
  is load-bearing, not bloat: long-lived second-touch residents do
  carry real reuse value on this trace, so a fixed Hot quota evicts
  faster than reuse distance. v003 showed that even the cycle-5
  insight's own two-fix S4LRU prescription cannot match CLOCK-Pro-
  lite — the fixes interact destructively (a fatter L3 starves the
  staging levels that gate promotion into it, and the L3 ghost ring
  almost never fires because few keys survive the three-hop climb).
- Combined with cycle 5's S4LRU regression, the champion's structural
  choices — ghost-anchored single-hop Cold→Hot promotion with an
  unbounded Hot pool — now have decisive structural credit across two
  cohorts: every alternative skeleton or hot-side budget cap tested
  has lost. The local optimum is structurally robust, not just
  plateau-by-default.
- Prescriptive guidance from a prior cycle-insight ("graft a ghost
  ring + top-heavy split") proved wrong when followed verbatim. The
  cycle-5 prescription was sound about *what was missing* from S4LRU
  but unsound about whether adding it would close the gap. Future
  cycle-insights should mark such prescriptions as conjectures rather
  than directives.
- The fitness-vector narrowness and evaluation-surface breadth flagged
  in cycles 1–5 remain unaddressed; this cycle's plateau break makes
  the breadth question more urgent, not less, because the gate's lift
  has been measured on exactly one trace and one capacity.

## Open questions

- Are there other latent precondition-collapse or measurement bugs in
  the champion lineage that mask real levers? The H01 result shows
  one such bug already concealed a working mechanism for a full
  cycle; the cohort design should now treat parity-at-champion as
  potentially diagnostic rather than terminal.
- Where does the new (post-gate) champion sit relative to the
  workload's actual ceiling on lirs:multi1@512? If it is close to the
  ceiling, single-trace tuning is exhausted; if not, are there other
  scan-admission refinements (sketch sizing, fingerprint width,
  two-counter promotion thresholds) worth a slot?
- Does the cold-admission gate's lift survive on other capacities of
  lirs:multi1 or on different traces, or is it tuned to this specific
  capacity-vs-scan-rate ratio?

## Continuation

Promote cycle-6 v001 (cold-admission gate, sketch owned by `put`
alone) as the new working champion — the first real hit-rate lift
since cycle 3. Drop two lineages outright: explicit Hot-pool capping
(v002 regressed; the unbounded Hot pool is load-bearing) and the
segmented-LRU skeleton in all forms tested so far (v003 confirmed
even the prescribed two-fix variant trails the plateau, on top of
cycle 5's symmetric loss). With the gate now confirmed real, narrow
refinements of the same lever — sketch sizing, fingerprint width,
counting-sketch instead of single-bit, gating threshold above 1 — are
worth at most one slot collectively, not a full cohort: cycle 3's
"diminishing returns on a single working knob" warning applies here
too. The highest-leverage move remaining is the breadth lever cycle 4
and cycle 5 both named: widen the evaluation surface (more
capacities of lirs:multi1, or additional traces sampled at matched
sizes) and/or address the fitness-vector narrowness, both so the
champion's gate-driven lift can be confirmed as general rather than
trace-specific and so structural variants that move axes other than
hit-rate can register at all. Treat any future prescriptive cycle-
insight guidance about specific structural fixes as a conjecture to
be tested in one slot, not a multi-fix recipe to be applied
verbatim — the cycle-5 S4LRU prescription failed exactly that test.
