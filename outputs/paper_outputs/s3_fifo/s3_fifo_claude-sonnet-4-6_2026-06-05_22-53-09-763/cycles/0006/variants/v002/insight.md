# Variant v002 Insight

## Helped

- hit-rate: Proactive repositioning of max-freq items to the head of `main` produced a negligible positive shift — the small fraction of items whose first saturation event occurred mid-queue were spared at most one eviction cycle, yielding an immeasurably small net rescue.

## Hurt

## Takeaway

Repositioning on first saturation does not move the needle on this workload: the eviction scan's existing second-chance mechanism already recycles high-frequency items effectively enough that physically relocating them to the head of `main` rescues essentially none that would otherwise have been evicted. The result indicates that the hot set on workload-a rarely reaches MAX_FREQ while simultaneously sitting close enough to the tail to be at eviction risk — the two conditions that must coincide for the repositioning to matter — making this structural change an expensive no-op rather than a protective mechanism.
