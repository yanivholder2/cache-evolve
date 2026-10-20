# Variant v003 Insight

## Helped

- hit-rate: Raising the admission bar to `incoming_freq > victim_freq.saturating_add(ADMISSION_MARGIN)` (margin 1) made a newcomer clear the probationary victim by two observations instead of one, so a handful of warm-but-not-hottest residents survived transient bursts and were hit again — a marginal upward nudge.

## Hurt

- (none observed): hit-rate is the only measured dimension and it did not regress; demanding a strictly larger margin only ever blocks admissions that would otherwise displace a resident, so the change cannot lose existing hits on this trace.

## Takeaway

A one-observation hysteresis margin in the eviction contest barely moves workload-a — only a couple of probationary keys are saved from a marginally-hotter, non-reused arrival. The strict tie-break was already close to optimal here, so the gain from protecting churn-survivors is real but tiny; widening the margin further would risk admitting too little, not too much.
