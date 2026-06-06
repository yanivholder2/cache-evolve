## Recurring wins

- Cheap, near-pure hot-path changes that *remove* work pay off: the
  inline integer hasher, segment-gated surgery-skips, and gentler ghost
  admission all sped or held latency by avoiding work rather than adding
  a branch.
- Orthogonal axes stack cleanly. Hit-rate policy (SLRU) and per-op-time
  (fast hasher) combined with neither paying for the other; the metadata
  re-encoding later folded onto that base the same way.
- The metadata win is real once casts are centralized and the segment
  flag is packed into a spare link bit — this crosses into the 24-byte
  node class while holding the hit-rate record and even shaving latency.
- Smarter *admission* (gated, evidence-based ghost re-entry) buys
  hit-rate for free, distinct from retention geometry.
- Modest protected-cap nudges (e.g. 4/5 to 7/8) recover hit-rate with no
  latency penalty.

## Recurring failures

- Representation changes billed as "free" still tax latency when they
  add per-access casts or sentinel encodings; only centralized/packed
  encodings escape the tax.
- Blunt protected-cap widening (large shares like 80→90%) lifts hit-rate
  but starves the protected-MRU fast path and regresses latency. The
  cost is magnitude-dependent, not categorical.
- Bare data-dependent hot-path guards (e.g. `tail == idx`) mispredict and
  regress both median and tail; the same skip wins only when gated on a
  cheaply predictable position.
- Latency micro-opts targeting non-bottlenecks lose: bounds-check elision
  and relink/store fusion removed work that was already free and paid
  real codegen damage instead.
- "Fusion" of correct list-surgery calls is not redundancy removal — it
  duplicates work plus a branch and perturbs codegen, regressing even
  untouched paths.

## Open questions

- Does the gated-ghost hit-rate gain actually reproduce on workload-a, or
  was it trace/size-specific? A near-zero recall delta warns it may not
  hold.
- How far can the protected fraction push past 7/8 before fast-path
  starvation reappears — is there a clean hit-rate-maximizing cap?
- With micro-opt and admission levers tapped, is the champion near this
  trace's latency floor, or does a structural change (hashing, traversal
  layout) still have room?

## Per-policy notes

- SLRU is the durable hit-rate base; its protected-MRU position is what
  makes the surgery-skip fast path safe, and what blunt cap-widening
  starves.
- Plain single-list LRU defeats the surgery-skip: the touched node is
  rarely the tail, so the guard mispredicts — keep the skip segment-gated.
- Ghost/reuse-distance history needs gentle admission (probation re-entry
  or repeated-hit gating); unconditional promotion to protected overshoots
  and drops hit-rate below pure SLRU while paying per-put bookkeeping.
- Admission gating is workload-contingent: short reuse distance on
  workload-a drains it, so admission cleverness is not a general win.
