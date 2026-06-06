# Variant v003 Insight

## Helped

(none — the W-TinyLFU window did not move any dimension favourably)

## Hurt

- metadata-size: Adding a `region: Region` discriminant to every `Node` pushes it from 48 to 56 bytes (alignment padding), and the dual-chain bookkeeping (`window_head/tail/len/cap` + `main_head/tail/len/cap`) inflates the `TinyLfuCache` struct from 152 to 216 bytes — pure structural overhead the single-chain parent avoided.
- per-op-time: Splitting one LRU into region-aware `unlink`/`push_back` with `set_head`/`set_tail`/`tail_of` match arms, plus the window-spill branch in `admit_not_full` and the candidate-vs-victim shuffle in `admit_full`, leaves p50 flat but worsens tail latency (get p99 ~79→147 ns, put p99 ~199→237 ns) from the extra branching on the eviction path.

## Takeaway

On workload-a at size 2048 the ~1% recency window (`window_cap = capacity/100`) failed to rescue cold-start hot keys: hit-rate stayed essentially flat (a marginal dip), while the per-node region flag and dual-chain machinery cost real metadata bytes and tail-latency. A window this small can't buffer enough newcomers to accumulate frequency before the gate, so W-TinyLFU's admission benefit doesn't materialize here — the partition only adds overhead.
