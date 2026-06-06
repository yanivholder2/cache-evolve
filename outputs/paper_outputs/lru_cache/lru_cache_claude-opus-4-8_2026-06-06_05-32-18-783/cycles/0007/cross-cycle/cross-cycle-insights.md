# Cross-Cycle Insights

## Recurring wins

- Changes that remove or avoid work on the hot path pay off; changes that
  add a branch or conversion to it do not. The integer hasher, the
  segment-gated surgery-skip, and gentler ghost admission all won by
  reducing work, never by adding a guard.
- Orthogonal levers stack cleanly when they touch disjoint structures:
  SLRU policy + fast hasher combined without either paying, and the
  metadata re-encoding folded onto the SLRU base the same way.
- The metadata win (narrow `u32` sentinel links, segment flag packed into
  a link's high bit, centralized cast helpers) reached the 24-byte node
  class while holding records — once casts were centralized rather than
  per-access.
- Mechanism is consistently cheap: spare-high-bit flags, in-struct stores,
  and pure layout re-encodings repeatedly added zero node bytes. The
  outcome is decided by the policy they encode, not the representation.
- Geometry headroom is real but narrow and magnitude-dependent: a modest
  protected-cap step (to 7/8) bought hit-rate for free where larger steps
  starved the fast path.

## Recurring failures

- Admission cleverness keeps losing on the short-reuse workload: two-hit /
  repeated-hit gates drain genuine repeaters out of probation before their
  second hit lands, dropping hit-rate while adding bookkeeping. Tried and
  regressed multiple times — stop revisiting on this trace.
- Blunt geometry moves (large protected-cap widening, 15/16 step) trade
  hot-path latency for hit-rate by starving the protected-MRU fast path. A
  sharp starvation cliff sits just above 7/8.
- Latency micro-opts targeting compiler-handled non-costs fail: bounds-check
  elision and relink/surgery "fusion" removed work that was already free
  (predicted-in-bounds accesses, store-buffer-absorbed writes) and paid real
  codegen damage instead.
- Position-gated surgery-skips only pay where the predicted position is
  actually common: a win inside the SLRU protected segment, a regression as
  a bare data-dependent branch or on single-list plain LRU.
- Unconditional ghost-to-protected re-admission overshoots, evicting hot
  keys and dropping hit-rate below pure SLRU.

## Open questions

- Is the standing champion at this trace's hit-rate/latency frontier now
  that geometry (top) and admission gating are both exhausted?
- Does a last-key get cache (attacks the named HashMap hash+probe cost)
  have merit? It targets a real hot-path cost but has died on infra twice,
  never measured.
- Does the gated-ghost hit-rate gain actually reproduce on the
  short-reuse workload, or was it trace/size-contingent?
- Where does remaining latency headroom live if compiler-redundancy edits
  are exhausted — structural hashing/traversal/link-surgery layout?

## Per-policy notes

- Segmented LRU (SLRU): the durable champion lineage. Hit-rate has two
  distinct levers — retention (protected geometry) and admission (what
  earns a slot); retention via a modest cap is the more reliable gain.
- Plain single-list LRU: the surgery-skip fast path does not generalize
  here — the touched node is rarely the tail, so the guard mispredicts.
- Ghost/reuse-distance admission: gentle admission (probation re-entry,
  repeated-hit gating) recovered signal once, but the gain has not
  reproduced on the short-reuse workload; unconditional promotion is
  refuted.
- Infra note: the metadata fold and the last-key cache each lost runs to
  disk-quota / simulator failures, not on merit — re-runs have repeatedly
  been warranted.
