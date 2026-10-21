# Variant v001 Insight

## Helped

- hit-rate: The hypothesis replaces the one-bit referenced flag with a two-bit saturating counter that is aged during victim scans. The evictor can then tell repeatedly hit entries from once-reused ones, and stale entries decay to zero and become victims. Hit-rate on workload-a rose over the parent, but only modestly, so the graded signal added less than the original move from position to reuse-based eviction.

## Hurt

- None measured. hit-rate is the only reported dimension. Aging rewrites counters on every scanned slot during eviction, which could cost CPU time that no dimension captures. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Going from "reused or not" to "how much reused, with aging" gave a real but small gain. Most of the benefit of reuse-aware eviction was already captured by the one-bit flag. The remaining headroom on this trace is probably not in finer eviction recency or frequency state. The result comes from a single trace and size, so the small gain could be noise-level on other workloads.
