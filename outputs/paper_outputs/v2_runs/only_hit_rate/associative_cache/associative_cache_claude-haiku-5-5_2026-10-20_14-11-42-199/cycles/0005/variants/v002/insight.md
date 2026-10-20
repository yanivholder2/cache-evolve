# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Hit-rate on workload-a fell slightly relative to the parent, so skipping the promotion when the protected half-bucket is full did not keep the protected working set shielded better than the forced demotion it replaced.

## Takeaway

Removing demote-on-promotion did not recover the hit-rate that v001 lost, and this variant ended slightly below its parent. The forced demotion therefore does not look like the main cause of the v001 loss; the protected cap itself, or the second-hit signal, is still a candidate. This is inferred from the hypothesis text and the two fitness vectors only: the variant's overlay source and manifest could not be read in this run, so the exact promotion-path change is not confirmed against the diff.
