# Variant v003 Insight

## Helped

*(no dimension improved)*

## Hurt

- hit-rate: Redirecting ghost population from small-queue cold evictions to main-queue evictions drastically reduces ghost-hit frequency, because items must survive the cold filter, earn a promotion to main, and then be second-chanced out before they can be tracked — a far rarer event than a cold small-queue rejection — leaving most re-accessed medium-popularity items without a fast-readmission path.

## Takeaway

The ghost queue's value in S3-FIFO comes primarily from its volume of useful signals, not the semantic "quality" of each signal. Small-queue cold evictions are plentiful and capture the exact case the ghost was designed for — items that return soon after being rejected by the cold filter — whereas main-queue evictions are sparse and describe items whose working-set residency has already largely expired. Elevating signal quality by feeding ghost exclusively from main evictions collapses ghost-hit coverage and forces a large share of returning items through the cold admission path repeatedly, producing a substantial hit-rate regression on this workload.
