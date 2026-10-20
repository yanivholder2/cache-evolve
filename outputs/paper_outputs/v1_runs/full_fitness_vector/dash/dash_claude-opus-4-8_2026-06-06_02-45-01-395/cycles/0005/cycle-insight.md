# Cycle 0005 Insight

## Cohort outcome

This cohort tried to bank the cycle-0004 wins by composition and to close
the last open hit-rate class: stacking v002's devirtualized hot path onto
v001's side-table parent (v001), monomorphizing the outer Bucket trait
object one layer further (v002), and adding a frequency-aware ghost-list
admission lever (v003). All three entered the Winners, but the cohort was a
clean sweep of confirmatory negatives — every promised stack either failed
to compose or regressed latency, and the new lever moved nothing.

## Patterns

- The two cycle-0004 Pareto wins are substitutive on a shared hot path, not
  additive. Porting devirtualization onto the side-table parent left get-p50
  pinned at the parent's floor, because the `target_heat` hash probe — not
  the vtable dispatch — is what bounds that variant's latency. You cannot
  collect both speedups while the side-table hash sits on the same get path.
- Structural devirtualization only pays when it removes indirection without
  fattening layout. Collapsing the *outer* Bucket trait object replaced a
  compact heap pointer per slot with a fat inline struct, widening the `Vec`
  stride and regressing every per-op metric — the inverse of the inner-policy
  win. The heap pointer was buying locality; removing it cost more than the
  saved dispatch.
- Hit-rate is now confirmed capacity/compulsory-capped against every class.
  The admission/frequency lever — the one untried family — left hit-rate
  bit-for-bit at the recency-only ceiling, buying only put latency from its
  ghost bookkeeping. Placement, gate, associativity, and now admission are
  all exhausted.

## Open questions

- With composition shown to be substitutive on the shared get path, is there
  a hot-path layout where the side-table relocation and the devirtualization
  stop competing — or is the metadata-floor corner and the latency floor
  simply mutually exclusive?
- Every remaining latency lever this cycle either fattened layout or added
  bookkeeping. Is there a structural change that removes indirection while
  *preserving or improving* locality, or has latency also saturated?

## Continuation

Treat hit-rate as fully closed — admission was the last class and it moved
nothing; spend no further slots probing it. Stop attempting to stack the
cycle-0004 size and latency winners on a single hot path: they are
substitutive, and outer-layer monomorphization regresses locality. The two
banked corners (v002 devirtualized latency baseline, v001 side-table size
corner) stand as the frontier; next cycle should look for a latency change
that improves locality rather than adds indirection or inline bloat, and
otherwise accept the frontier may be converged.
