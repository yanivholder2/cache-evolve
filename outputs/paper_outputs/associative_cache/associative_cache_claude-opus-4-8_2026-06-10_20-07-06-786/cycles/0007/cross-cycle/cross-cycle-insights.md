# Cross-Cycle Insights

## Recurring wins

- Explicit segmentation beats scalar tuning: demanding proof of reuse before
  granting protection (promote-on-second-hit, then promote-on-first-hit into a
  protected segment) is the foundation every later baseline builds on.
- Earned protection is load-bearing. Schemes that hand rising keys a full
  probation lap to promote — FIFO/oldest-first probation, promote-straight-to-MRU
  placement — consistently win; the durable protection they confer is the
  source of hit-rate on this trace.
- Geometry is a live axis when behavioral logic saturates: one capacity-conserving
  bucket-count halving (wider associativity) delivered the only hit-rate gain after
  several flat cycles.
- Behavior-neutral O(1) rewrites of linear victim/eviction scans are safe, real
  cost-side wins — though under hit-rate-only fitness they always land as ties.

## Recurring failures

- Protection erosion is the dominant failure mode across the whole run. Every
  variant that withholds or strips earned protection regresses: counter aging,
  raised promotion thresholds (two-hit), protected-segment caps with demotion,
  CLOCK/one-bit recency approximation, LFU-min fallback for protected eviction,
  and ghost-ring accelerated re-entry. The trace is collision-bound and rewards
  "once-hot immortality," not staleness sensitivity.
- The two early orthogonal levers do not compose — front admission plus
  frequency-first eviction conflict, making a fresh key simultaneously
  recency-oldest and minimum-frequency, i.e. the immediate victim.
- Pushing geometry past one halving fails from both sides: a second halving
  over-widens and scatters hot keys; an absolute associativity target degenerates
  to a fixed sub-optimal width. The single-halving setpoint is bracketed and settled.

## Open questions

- Is hit-rate now at its ceiling? Admission, promotion, protected-eviction signal,
  re-entry, and bucket width have all been exhausted.
- Cost-side wins keep recurring and keep tying invisibly. Is a per-op-time fitness
  dimension warranted so behavior-neutral savings can be selected rather than tie?
- The one largely untested hit-rate lever is looser/larger protected sizing and its
  interaction with the settled wider-bucket geometry — do the two axes stay independent?

## Per-policy notes

- Pure LRU / recency: full positional ranking matters in small buckets; coarse
  one-bit (CLOCK) reprieves underperform. Recency at the protected end is not
  safely approximated away.
- LFU / frequency: useful only as the original victim-selection signal; as a
  fallback or banked count it fails to separate hot from rising keys, and aging
  it removes the immortality that makes it valuable.
- Segmented-LRU (current baseline): the strongest scheme — single-hit promotion to
  MRU, FIFO/oldest-first probation, uncapped protected segment. Admission/promotion/
  eviction logic is repeatedly confirmed saturated for hit-rate.
- Geometry / associativity: orthogonal to bucket logic; exactly one capacity-conserving
  halving is optimal, with routing loss dominating beyond it. Wider buckets raise
  per-eviction scan cost, recoverable via O(1) rewrites.
