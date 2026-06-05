# Variant v003 Insight

## Helped

- hit-rate: Stash hits bypass the target tier and land directly in the probing (hot) bucket, avoiding an extra tier traversal and reducing competition in target for newer items, enabling faster recycling of proven stash-resident items back into the hot tier.

## Hurt

(none observed)

## Takeaway

Fast-tracking stash hits directly to the probing bucket improves hit rate by eliminating unnecessary tier traversal. Items that have already proven themselves (by reaching the stash and being accessed again) jump immediately to the hot tier, where they enjoy priority against newer arrivals still climbing through the target tier. The one-more-chance demotion logic correctly handles foreign residents, maintaining the tier invariant without special-casing stash-sourced promotions.
