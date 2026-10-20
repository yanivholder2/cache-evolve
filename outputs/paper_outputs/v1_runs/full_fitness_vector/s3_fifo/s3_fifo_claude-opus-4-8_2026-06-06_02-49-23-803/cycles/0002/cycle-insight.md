# Cycle 0002 Insight

## Cohort outcome

This cycle tested three capacity/retention knobs on workload-a, branching
mostly off cycle 0001's split-ratio winner: pushing the small/main split
narrower, widening the ghost queue, and raising the frequency saturation
ceiling. The two retention-respecting moves — a narrower admission filter
and a taller freq cap — both lifted hit-rate, while widening the ghost
queue regressed it and grew the control struct.

## Patterns

- The split sweep remains the most reliable lever: continuing to move
  capacity from the admission filter into main raised hit-rate again, but
  the gain is shrinking — the filter is nearing the point where it stops
  screening one-hit-wonders.
- Capacity knobs are **not additive**: stacking a wider ghost on the
  winning split *lowered* hit-rate. Widening reuse-distance detection
  readmitted stale long-distance keys that displaced recently-reused
  entries — this trace rewards near reuse, not far reuse.
- Rewarding *demonstrated* reuse (a higher freq ceiling for already-hot
  keys) is a small, real gain and is orthogonal to admission: it preserves
  the load-bearing strict promotion/ghost thresholds while only deepening
  durability for keys that earned it. This confirms cycle 0001's lesson —
  reward proven reuse, never loosen the filter for weakly-touched items.
- Metadata-size is a live cost: adding a control field (ghost sizing) grew
  the struct for no benefit. Prefer knobs that retune existing fields over
  ones that add state.
- Per-op-time deltas again read as run-to-run noise; no change altered the
  resident scan size, and the freq-ceiling change can only lengthen the
  eviction loop, not shorten it.

## Open questions

- Where exactly does the split stop paying off — is the next step past
  capacity/16 the one that finally under-screens one-hit-wonders?
- Is the freq-ceiling gain still positive when combined with the narrower
  split, or do the two retention/capacity levers interfere like split and
  ghost did?

## Continuation

Keep sweeping the small/main split but expect diminishing returns and
watch for the crossover where the filter under-screens — find the knee,
don't assume monotonicity holds. Drop ghost enlargement: longer reuse
distance is not rewarded on this workload, and it costs metadata. The
freq-ceiling lever is worth one more probe, ideally *combined* with the
narrow split, to test whether demonstrated-reuse durability and
capacity reallocation are genuinely orthogonal. Continue treating
per-op-time as noise unless the resident scan size changes.
