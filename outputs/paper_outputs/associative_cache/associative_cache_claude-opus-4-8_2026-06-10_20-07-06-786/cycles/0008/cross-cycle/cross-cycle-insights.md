# Cross-Cycle Insights

## Recurring wins

- Explicit **segmentation** beats tuning any single frequency scalar:
  separating a probationary tier from a protected tier, with promotion
  gated on demonstrated reuse, is the strongest scan-resistant scheme found
  and has anchored the baseline since it appeared.
- **FIFO / oldest-first probation** outperforms newest-first churn: giving a
  fresh key a full lap to earn promotion recovers rising keys that would
  otherwise be discarded before their second reference.
- **Durable protection ("once-hot immortality")** is load-bearing on this
  trace. Every scheme that keeps proven-hot keys protected indefinitely, at
  the current protected size, has won or held; every scheme that makes
  protection harder to earn or easier to lose has regressed.
- **Promote-straight-to-MRU** placement is settled; seating promotions lower
  in the protected segment regresses.
- A single **capacity-conserving geometry change** (halving bucket count,
  doubling bucket size) lifted hit-rate via associativity alone — the one
  hit-rate lever found outside admission/promotion/eviction logic.
- Behavior-neutral **O(1) / reorder-eliding cost rewrites** are always safe to
  adopt; they never harm hit-rate.

## Recurring failures

- **Any erosion of earned protection** regresses, repeatedly and for the same
  reason (collision-bound trace): counter aging, front-admission composition,
  two-hit promotion thresholds, protected caps with demotion, elite/third
  tiers, and reserved-probationary slots have all failed.
- **Recency approximation** (CLOCK one-bit reference) underperforms full
  positional ranking in small buckets.
- **Re-entry / re-signal shortcuts** weaken discrimination: ghost-ring fast
  re-entry and LFU-min eviction fallback both let weakly-warranted keys seize
  protected space and regressed.
- **Geometry past one halving** overshoots in both directions — a second
  halving scatters hot keys; an absolute associativity target degenerates to a
  narrower fixed width. One halving is bracketed and optimal.
- **Routing/hash-quality** improvements buy nothing — the default hash is
  already effectively uniform on these keys.
- The orthogonal Cycle-0001 levers (frequency victim-choice + front admission)
  **do not compose** — they conflict.

## Open questions

- Is hit-rate now at its ceiling? Admission, promotion, eviction, re-entry,
  protected sizing/tiering, routing, and bucket width have each been ruled out.
- Should a **per-op-time fitness axis** be added? Cost-side wins keep recurring
  and keep landing as undifferentiated ties, invisible to hit-rate-only
  selection.
- Is there an untested **trace-structure / workload-regime** axis (e.g. a
  shifting working set) where staleness would dominate, or is this trace firmly
  collision-bound?
- Is the protected-segment cap permanently dead code on this trace, or does any
  regime saturate it?

## Per-policy notes

- **Segmented-LRU (current lineage):** the durable baseline. Strongest on
  scan resistance; saturated on hit-rate — admission, promotion, and eviction
  logic all repeatedly probed without gain.
- **FIFO probation:** the winning probation order; oldest-first promotion
  window beats newest-first eviction.
- **Pure LFU / frequency:** useful only as the original victim signal; aging,
  LFU-min fallback, and letting `==1` newcomers into the tally all erode the
  hot/rising distinction and regress.
- **LRU (plain):** protected-end recency ranking beats banked frequency and
  coarse one-bit (CLOCK) approximations for choosing which protected key to
  drop.
- **Geometry/associativity:** independent live axis; exactly one
  capacity-conserving halving is optimal, bracketed on both sides.
