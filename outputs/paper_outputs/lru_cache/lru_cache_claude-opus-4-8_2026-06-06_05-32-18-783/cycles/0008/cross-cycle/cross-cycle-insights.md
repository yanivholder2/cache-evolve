## Recurring wins

- Cheap, near-pure hot-path edits pay off: the inline integer hasher
  was a clean per-op-time win early and keeps stacking for free.
- Mechanism/representation changes are genuinely free when they *reduce*
  work rather than add a branch — the narrow-link metadata fold to the
  24-byte node class held hit-rate and even shaved latency once casts
  were centralized and the segment flag was packed into a link's spare
  high bit.
- Orthogonal axes stack cleanly: segmentation policy and the fast hasher
  combined with neither paying for the other, holding both records at
  once. Disjoint structures (node layout vs. external store) fold well.
- Modest geometry tuning is free where micro-opts are not: a one-eighth
  protected-cap nudge bought hit-rate with no latency cost.
- The mechanism-vs-policy split holds across nearly every cycle: layout
  changes that keep the node byte-for-byte are safe; the wins and losses
  come from behavior, not representation.

## Recurring failures

- Admission cleverness on this short-reuse trace is tried-and-regressed
  many times over: two-hit/warm probation gates drain genuine repeaters
  before their second hit, and ghost/reuse-distance re-admission floods
  protected and evicts hot keys. The one-cycle hit-rate gain from gated
  admission proved trace/size-contingent and did not reproduce.
- Latency micro-opts that attack compiler-handled or store-buffered
  non-bottlenecks lose every time (bounds-check elision, relink fusion,
  probe-skipping get caches). They trade a non-cost for real codegen
  damage. The HashMap probe is settled as *not* the get bottleneck.
- The surgery-skip fast path is segment-local: it wins inside protected
  (touched node is reliably MRU) but mispredicts on single-list LRU or
  as a bare data-dependent branch.
- Blunt/large protected-cap widening starves the fast path; per-access
  width casts tax latency. Big steps and naive encodings are dead ends.

## Open questions

- Is the standing champion at this trace's hit-rate/latency frontier, or
  does the one never-directly-attacked cost — link surgery and cache-line
  traversal during promote/demote — still hold room?
- Should growth of the cache struct's own footprint be treated as a
  constraint? Footprint growth alone repeatedly perturbed put p50 even
  when the code was byte-identical, a cost the scored metadata-size never
  captures.

## Per-policy notes

- Segmented LRU is the durable champion family. Its protected geometry
  has a narrow free window: below ~4/5 leaves hit-rate on the table, the
  ~7/8 step is free, and ~15/16 hits a sharp starvation cliff. Headroom
  above 7/8 is effectively closed from the top.
- Ghost/reuse-distance admission: refuted on the short-reuse workload —
  costs both hit-rate and latency; stop revisiting.
- Plain single-list LRU: the protected-segment surgery-skip does not
  transfer here; keep it segment-gated only.
- Fast integer hasher: a free, stackable win that underlies the champion.
