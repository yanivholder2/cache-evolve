# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Gating probation-to-protected promotion on the sketch estimate, instead of promoting on every hit, means a second hit no longer guarantees protection, so entries that v002 would have promoted can stay in probation where they remain exposed to eviction before they are confirmed as hot.

## Takeaway

On workload-a at size 2048, making promotion conditional on the sketch estimate beating the protected tail was a slight step back from v002's unconditional promotion. The hypothesis assumed frequency would separate genuinely hot entries from one-off re-references, but the estimate is a lagging count and does not capture recency, so the gate appears to reject some entries that were about to earn a second hit. The source diff could not be inspected from this variant's directories, so the attribution to the promotion gate rests on the hypothesis and the fitness delta, not on confirmed code.
