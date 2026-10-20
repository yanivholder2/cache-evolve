# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The ghost gate refuses every key's first miss into a full bucket, so a key with genuine reuse loses a whole miss cycle before the admission comparison can even see it, and the one-capacity ghost window forgets keys whose second access arrives later than that, forcing them back through the refusal.

## Takeaway

Stacking a first-miss refusal in front of the shared-frequency admission gate did not add the recurrence information the hypothesis expected: the counter comparison already bypasses one-shot keys whose frequency does not beat the victim's, so the ghost list mostly delays admission for keys that would have been admitted. Any future recurrence signal probably needs to replace the counter comparison's rejection rather than sit in front of it.
