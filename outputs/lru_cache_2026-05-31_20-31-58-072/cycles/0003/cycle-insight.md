# Cycle 0003 Insight

## Cohort outcome

All three variants tuned the cycle-2 CLOCK-Pro-lite champion along
orthogonal axes — ghost-ring size (v001), Hot-demotion stickiness
(v002), and ghost provenance plus warm re-admission (v003) — and only
v001 landed on the Pareto front, and only by a marginal lift. The two
policy-shape variants were inert: their machinery sits on a code path
that this trace/size barely exercises.

## Patterns

- The champion's loss channel on lirs:multi1@512 is concentrated on
  pass 1 (Cold sweep) of the eviction loop, not pass 2. v002 and v003
  both edited pass-2 / Hot-side mechanics (sticky demotion, Hot-ghosts,
  warm re-admit) and produced exactly the parent's score — pass 2
  simply does not fire often enough at this cap for those knobs to
  bite.
- Among the three knobs, the only one that moved the needle was
  monotonic-magnitude tuning on a mechanism already in the hot path
  (ghost length). Even there, quadrupling the ring caught only a
  sliver, indicating the re-reference distribution past the
  quarter-capacity mark has a thin tail.
- The cohort confirms cycle-2's reading from the other side: the
  champion's win came from the *existence* of a ghost and a
  promotion class, and further marginal returns on the same workload
  are small. We are climbing toward a local plateau on this trace.
- The fitness-vector bottleneck flagged twice now remains unaddressed;
  the cohort again competed only on hit-rate.

## Open questions

- If pass 2 is dormant at cap=512, would Hot-side knobs (H02-style
  sticky demotion, H03-style Hot-ghosts) actually pay off at smaller
  capacities or on traces where the Cold pool runs out and pass 2 is
  forced to fire?
- What are the pass-1 levers we have not yet pulled — Cold-admission
  filtering, ghost-hit promotion targeting, or alternative
  scan-resistance heuristics on the Cold sweep itself?
- Is the lirs:multi1 trace itself now the limiting factor — i.e. how
  much more hit-rate is recoverable on this single workload before
  diversity of traces becomes the bottleneck instead of policy?

## Continuation

Treat v001 (full-capacity ghost ring) as the new working champion but
flag that the ghost-sizing knob is in strong diminishing returns;
further size sweeps are not worth a slot. Drop the Hot-side
modification lineage (H02 sticky demotion, H03 Hot-ghosts, hot_credit)
on this trace/size — three variants now show pass 2 is dormant here,
so any pass-2-only edit will keep scoring at parity. Direct next-cycle
hypothesis-writers to pass-1 / Cold-side mechanics instead: how Cold
victims are chosen, how ghost-hits seed Hot state on re-admission, or
Cold-admission gating that filters scan inserts before they pollute
the ring. The fitness-vector narrowness (no per-op-time, no
metadata-size) remains the unaddressed meta-bottleneck and should be
named again as a constraint; the cohort is otherwise about to plateau
on hit-rate-of-one-trace.
