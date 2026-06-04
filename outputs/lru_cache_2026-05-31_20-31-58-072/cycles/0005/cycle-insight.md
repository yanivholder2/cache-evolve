# Cycle 0005 Insight

## Cohort outcome

The cohort tried one genuinely new direction the cycle-4 cycle-insight
called for — a full skeleton swap to S4LRU (v003) — alongside two more
single-knob edits to the CLOCK-Pro-lite champion (cold-admission
bypass-on-first-sight in v001, ghost-ring-position-weighted promotion
in v002). The two single-knob edits tied the champion at parity (so
they sit on the Pareto front by tie); the skeleton swap regressed
below the plateau. The working champion is unchanged.

## Patterns

- The plateau flagged in cycle 4 held a second time: no variant moved
  hit-rate up. Three consecutive cohorts on this trace/size have now
  failed to lift the scored axis.
- The one variant that swapped skeletons (S4LRU) lost ground. A
  multi-hop promotion ladder paired with conservation-by-demotion
  shrinks effective top-level capacity on a trace whose re-reference
  distance does not survive three intermediate hops — recency
  segmenting *without* a ghost ring or an asymmetric (top-heavy) split
  is strictly worse than the current ghost-anchored skeleton on
  lirs:multi1@512. This is the first cycle that gives the champion
  structural credit, not just plateau credit: the ghost ring and the
  single-hop Cold→Hot promotion are doing real work.
- Ghost-ring *position* (v002) is not a usable freshness gradient
  here: freshest-quarter ghost-hits behaved identically to older ones.
  Differentiated promotion strength keyed on ghost position is a dead
  knob on this workload.
- v001's parity is a false negative, not a verdict on the lever: the
  first-seen sketch was marked on `get` *before* `put` read it, so
  under the framework's miss-then-put pattern the gate's precondition
  was already false by the time it could fire. Cold-admission gating
  itself — the lever the cycle-3 cycle-insight named — was therefore
  not actually exercised this cycle.
- The fitness-vector narrowness (no per-op-time, no metadata-size)
  flagged in cycles 1–4 remains unaddressed, and is now joined by the
  related unaddressed lever from cycle 4: widening the evaluation
  surface to other capacities or other traces. Both calls went
  unanswered this cycle.

## Open questions

- Would a *correctly instrumented* cold-admission gate (sketch marked
  only on `put`, or the gating decision threaded into `get`) actually
  lift hit-rate, or would it too come back at parity once the
  precondition-collapse bug is fixed?
- Is any non-CLOCK-Pro-lite skeleton viable on this trace, or does
  every alternative need at least the ghost ring and the single-hop
  promotion to keep up? Would S4LRU with an asymmetric top-heavy split
  or a ghost ring grafted on close the gap?
- After three plateau cohorts and one structural regression, is the
  loop's productive bottleneck still policy design at all, or has it
  decisively shifted to evaluation-surface / fitness-axis breadth?

## Continuation

Keep cycle-3 v001 (full-cap ghost ring) as the working champion. Drop
two lineages outright: ghost-ring positional weighting (v002 confirmed
inert), and unaugmented multi-level segmented LRU (v003 confirmed
below the plateau — do not retry S4LRU without either an asymmetric
top-heavy split or a ghost-ring augmentation). The one single-trace
policy lever still genuinely unpulled is cold-admission gating: the
H01 mechanism was not actually exercised because of a sketch-marking
bug, so a corrected re-attempt (mark sketch only on `put`, or thread
the gating decision through `get`) is worth exactly one slot — not
more. Beyond that one retry, hypothesis-writers should stop spending
slots on further single-trace, single-capacity refinements and finally
act on cycle 4's directive: widen the evaluation surface (more
capacities of lirs:multi1 or additional traces, sampled at matched
sizes) and/or address fitness-vector narrowness so structural changes
can be scored on axes other than hit-rate. With one cohort having now
confirmed the champion's structural choices are load-bearing (S4LRU
lost), evaluation-surface breadth is the highest-leverage remaining
move.
