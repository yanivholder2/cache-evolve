# Cycle 0001 Insight

## Cohort outcome

All three variants targeted distinct structural axes — sketch aging, node memory layout, and hash computation — but none produced a net winner; each achieved its primary structural goal while incurring a regression that cancelled the benefit.

## Patterns

- **Structural goals met, performance goals missed across the board.** Decay halved counters as intended, node compaction cut metadata size by ~50%, and double-hashing reduced hash invocations — yet all three regressed on at least one fitness dimension, suggesting the baseline is already well-tuned for the cost-sensitivity of this workload and key type.
- **The workload's stable working set invalidated the decay premise.** The LIRS trace's long-lived hot set makes accumulated frequency a reliable admission signal; an aggressive reset interval destroyed that signal faster than accesses could rebuild it, inverting the expected hit-rate gain.
- **Compiler-generated code for idiomatic Rust outperformed explicit arithmetic optimizations.** Both the `Option<usize>`→`u32` sentinel swap and the double-hash serial chain discovered that the compiler's branch-prediction and ILP-friendly codegen for standard abstractions beat hand-rolled alternatives at this scale.
- **Size and speed moved in opposite directions.** Compact node representation improved memory density but doubled hot-path latency, revealing a trade-off that cannot be resolved by layout alone.

## Open questions

- Is there a decay interval (much longer than `width/2`, or entropy-triggered) that preserves the frequency signal on stable working sets while still aging stale counts?
- Can node compaction recover its speed loss if `NonZero<u32>` restores niche optimization and eliminates explicit sentinel comparisons?
- Would a single-pass multiply-shift or seeded hash avoid the serial `h1→h2` data dependency while still reducing total hash work per sketch call?

## Continuation

Next cycle should deprioritize the `width/2` decay cadence — it is clearly too aggressive for this trace — and instead either experiment with much longer or adaptive intervals, or focus on a different axis entirely. The u32 node compaction trade-off is worth one more attempt with `NonZero<u32>` before abandoning; if that also regresses latency, drop the layout approach. For hash reduction, any new hypothesis must eliminate serial inter-hash dependencies, as ILP loss dominates the gain from fewer hash calls on cheap integer keys.
