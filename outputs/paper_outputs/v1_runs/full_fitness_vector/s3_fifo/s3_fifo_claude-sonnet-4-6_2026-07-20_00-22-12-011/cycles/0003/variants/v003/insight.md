# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: Lowering the promotion threshold to `freq > 0` admits once-warm keys (exactly one re-access while in small) directly into main, bypassing the ghost window — but these keys enter main with only `freq = 1` and are therefore weak occupants, causing faster main eviction churn that displaces entries with stronger recency evidence than the ghost filter would have admitted.

## Takeaway

The ghost window in S3-FIFO is not just a latency buffer for cold keys — it acts as a second demand filter for once-warm keys. When a `freq = 1` entry is ghosted under the original `freq > 1` rule, it only re-enters main if explicitly re-requested, meaning main receives a confirmed-recurring key. Promoting immediately at `freq > 0` short-circuits that confirmation step, filling main with keys whose recency signal is weaker, and the resulting eviction pressure slightly erodes hit-rate on this Zipfian trace. The paper's `freq > 1` threshold appears calibrated to preserve the ghost channel as a quality gate, not merely a frequency gate.
