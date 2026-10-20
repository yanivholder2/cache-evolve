# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Periodically halving every count-min counter once `total` reaches `10x capacity` discards accumulated frequency history, so genuinely hot keys lose the estimate margin they need to beat LRU victims at admission, and the admission filter starts rejecting/replacing on noisier post-reset counts.

## Takeaway

On this workload the popularity distribution is stable, not shifting, so the aging the hypothesis bet on solves a problem this trace doesn't have — and the `10x capacity` reset threshold ages aggressively enough that it throws away the long-term frequency signal the baseline relies on, lowering hit-rate rather than raising it. Sketch decay is only worth keeping if paired with a workload that actually exhibits popularity churn or a much larger sample_size.
