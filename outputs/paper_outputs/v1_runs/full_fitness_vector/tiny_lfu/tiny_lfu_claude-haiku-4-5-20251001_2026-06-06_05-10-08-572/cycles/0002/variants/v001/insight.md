# Variant v001 Insight

## Hurt

- **hit-rate**: Relaxing admission from `incoming_freq > victim_freq` to `incoming_freq >= victim_freq` admitted more borderline items into the cache, degrading hit-rate as these marginal admissions displaced more valuable frequent items despite the wider sketch's improved frequency estimates.

## Helped

(None)

## Takeaway

Relaxing the admission barrier to be more permissive actually hurt hit-rate, contradicting the hypothesis that wider sketch accuracy and looser admission would synergize. The wider frequency estimation doesn't overcome the fundamental issue that lowering the bar admits items that are not hot enough to justify evicting better-established cache residents—the threshold matters more than marginal frequency improvements for admission decisions on this workload.
