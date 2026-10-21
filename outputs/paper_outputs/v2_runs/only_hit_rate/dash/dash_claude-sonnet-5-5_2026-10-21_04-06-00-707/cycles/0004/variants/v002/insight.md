# Variant v002 Insight

## Helped

- None measured. hit-rate is the only reported dimension, and it did not improve.

## Hurt

- hit-rate: The hypothesis made the second-sighting filter conditional on the victim. A first-sighting key is admitted at once when the cold-end victim has never been hit, and filtered only when the victim has been hit. This reopened the door to one-hit-wonders, which the parent's unconditional filter had been blocking. Hit-rate on workload-a fell below the parent's. On this trace, "victim never hit" is a poor sign that the new key is worth admitting. Entries that have not been hit yet are often still-warm entries that would be reused, and replacing them with unproven keys costs more than it gains.
- Unmeasured: the per-slot hit bit, if one had to be added, and the victim inspection on the miss path were not evaluated. No metadata or per-op-time dimension was reported. I could not read the overlay sources, so the diff is inferred from the hypothesis text.

## Takeaway

The parent's gain came from applying the second-sighting check unconditionally. Exempting victims that have not been hit gave back part of that gain, so "this victim has not been hit yet" is not a reliable signal that evicting it is cheap. Any refinement of the filter needs a stronger victim-value signal than a single hit bit, or should leave the filter unconditional.
