# Variant v002 Insight

## Helped

- (none): the only measured dimension, hit-rate, did not improve.

## Hurt

- hit-rate: Shrinking `protected_cap` from `capacity * 4 / 5` to `capacity / 2` makes the protected segment hold fewer re-referenced keys. Hot keys that were promoted get demoted back into probation and evicted sooner. On workload-a the extra probation space did not make up for that loss.

## Takeaway

On this workload the hypothesis was wrong. Giving probation more room to catch second hits is worth less than keeping a large protected segment for the proven-hot set. The 80% split in the parent is closer to the right balance, and the useful direction is probably at or above 80%, not below it. I could not read the overlay source directly, so this reading relies on the hypothesis file and the fitness vectors.
