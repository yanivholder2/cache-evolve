# Variant v003 Insight

## Helped

## Hurt

## Takeaway

The evaluation shows hit-rate unchanged from parent v002, and metadata-size is not measured at all, so this variant does not yet show whether moving the "seen in stash" marker into a spare tag or bitmap bit kept entry size at baseline. The unchanged hit-rate is consistent with the admission rule being preserved, but it is also what the hypothesis's "no spare bit, make no change" fallback would produce, so the diff needs to be checked to tell which case happened.
