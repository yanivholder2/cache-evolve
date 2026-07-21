# Variant v002 Insight

## Helped

- per-op-time: The MRU-guard added to `get()` eliminated up to six pointer writes for keys that are already at the tail, collapsing the get p50 latency dramatically — exactly the hot-item pattern the hypothesis targeted.

## Hurt

- per-op-time: The extra branch in the non-MRU promote path pushed get p99 higher, and put p50 also regressed despite `put()` being unchanged — likely from altered instruction-cache pressure across both code paths during the benchmark.

## Takeaway

Guarding `get()` to skip `unlink`+`push_back` when the node is already the MRU tail delivers a large median-latency win on repeated accesses to the same key, confirming the hypothesis's core claim. However, the optimization is not a clean sweep: the added conditional raises tail latency on the promote path and appears to disturb branch-predictor or instruction-cache state enough to degrade `put` throughput. The change is a strong get-p50 optimization but should be weighed against its p99 and put-side costs before accepting.
