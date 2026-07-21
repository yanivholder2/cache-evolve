# Cross-Cycle Insights

## Recurring wins

- No multi-cycle wins to report yet — only one cycle completed. Patterns will emerge once successful approaches replicate across cycles.

## Recurring failures

- **Aggressive sketch decay (width/2 interval) degrades hit rate on stable working sets.** A working set with long-lived hot entries accumulates reliable frequency signal; resetting too early destroys that signal faster than accesses rebuild it, producing a net regression in admission quality.
- **Explicit arithmetic optimizations underperform idiomatic Rust abstractions.** Both sentinel-value compaction (`Option<usize>` → `u32`) and a double-hash serial chain lost ground to compiler-generated code for standard patterns; the compiler's branch-prediction and ILP-friendly codegen proved difficult to beat manually at this scale.
- **Memory-layout improvements and hot-path latency move in opposite directions.** Compact node representations cut metadata size substantially but doubled latency on the hot path, revealing a size/speed tension that layout alone cannot resolve.

## Open questions

- **What decay cadence preserves frequency signal on stable traces?** The width/2 interval is too aggressive; a much longer or entropy-triggered schedule might age stale counts without destroying reliable signal.
- **Does `NonZero<u32>` recover the latency lost by node compaction?** Niche optimization and elimination of explicit sentinel branches could close the gap — worth one more attempt before abandoning the layout axis.
- **Can hash work be reduced without serial inter-hash dependencies?** A single-pass multiply-shift or seeded hash could avoid the h1→h2 data-dependency chain that erased the gain from fewer hash calls on cheap integer keys.

## Per-policy notes

- **TinyLFU (LIRS trace):** The workload's stable hot set makes accumulated frequency a strong admission signal. Any hypothesis that resets or disrupts frequency counts at short intervals is likely to regress hit rate. Structural changes to node layout or hash computation must not introduce serial hot-path dependencies, as this workload is latency-sensitive and ILP loss dominates small arithmetic savings.
