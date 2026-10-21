# Variant v002 Insight

## Helped

(none — the only dimension measured for this variant, hit-rate, moved the wrong way; metadata-size and per-op-time were not reported in v002's fitness vector, so we can't confirm whether they changed.)

## Hurt

- hit-rate: Halving every counter after `10 × capacity` conservative-update increments pushes most sketch estimates down to 0–1. As a result, the strict-`>` admission contest against the LRU victim either ties, so the victim stays, or is decided by one recent touch. The sketch loses the long-run frequency signal that the baseline's ever-growing counters gave it on workload-a. Without a window or doorkeeper in front, there's nothing to make up for that lost history.
- hit-rate: Conservative update also lowers the estimates of genuinely warm keys (only the minimum cells are bumped). Combined with aging, warm residents and cold newcomers end up with similar low counts. That blurs exactly the distinction this TinyLFU's admission policy depends on.

## Takeaway

On this trace, the baseline's "stale hot keys can never be displaced" problem cost less than the precision that decay removed. Aging at a 10×capacity sample size made the frequency signal too short-lived and coarse for a plain LRU + strict-`>` admission filter. The variant changed two things at once (conservative update and halving), so this result can't say which one caused the regression. A follow-up should test them separately, and should try aging with a much larger sample size or a tie-breaking rule, rather than drop the idea of decay altogether.
