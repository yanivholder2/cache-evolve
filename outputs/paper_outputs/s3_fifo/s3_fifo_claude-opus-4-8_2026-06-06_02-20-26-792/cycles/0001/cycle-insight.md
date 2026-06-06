# Cycle 0001 Insight

## Cohort outcome

This cycle probed three independent S3-FIFO levers against the baseline —
admission threshold (H01), small/main partition ratio (H02), and ghost-queue
history depth (H03). Only the partition-ratio change (v002) landed in the
Winners; loosening the promotion gate and deepening ghost history both hurt
hit-rate.

## Patterns

- The three "move toward the paper's defaults" hypotheses split sharply: the
  sizing default helped, while the paper's promote-on-any-reuse threshold and a
  wider history window both regressed. On this LIRS workload the baseline's
  stricter filtering is load-bearing, not an oversight.
- Anything that admits more keys into the main resident queue — looser
  promotion (v001) or long-gap direct re-admission via deeper ghost (v003) —
  displaced hotter residents and cost more than it saved. The main queue is
  contention-sensitive; protecting it beats feeding it.
- Conversely, giving the main queue *more capacity* (v002) was the one net win,
  but only modestly, suggesting the partition ratio is a real but bounded knob.

## Open questions

- How far can the small/main ratio be pushed before the shrinking admission
  filter starts leaking one-hit-wonders into ghost/main? v002's gain was small —
  is there a sharper optimum past ~10/90?
- v003 showed history depth and the direct-to-main re-admission rule are
  coupled. Would deeper ghost history help if ghost hits were gated (e.g.
  re-admitted via small queue, or only when fresh) rather than pushed straight
  to main?

## Continuation

Pursue the partition-ratio lever further — sweep small/main beyond 10/90 to
find where the main-queue gain reverses. Drop standalone threshold-loosening
(H01-style) and naive ghost-deepening (H03-style); both regressed because they
crowd the main queue. If revisiting ghost depth, pair it with a gate on the
direct-to-main re-admission path rather than widening history alone.
