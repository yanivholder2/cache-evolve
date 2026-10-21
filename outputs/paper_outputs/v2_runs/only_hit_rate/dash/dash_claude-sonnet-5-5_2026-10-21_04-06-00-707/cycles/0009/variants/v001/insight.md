# Variant v001 Insight

## Helped

- None measured. The change cut ghost-table writes by skipping cold victims, but no reported dimension captures write or memory churn. I could not list or read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Hurt

- hit-rate: The hypothesis stops recording ghost entries for victims whose counter was below 2. It also caps the restored counter at 2. Cold keys that return after eviction therefore get no head start, and hot ones get less than the saved value would give. Hit-rate on workload-a slipped slightly below the parent.

## Takeaway

The parent's ghost-table gain came partly from remembering cold victims, not only hot ones. Filtering by saved heat removed useful history and so did not work as a pure noise reduction. The change is tiny and measured on one trace and one size, so treat it as a mild negative signal. Further tightening of ghost admission looks unpromising on this workload.
