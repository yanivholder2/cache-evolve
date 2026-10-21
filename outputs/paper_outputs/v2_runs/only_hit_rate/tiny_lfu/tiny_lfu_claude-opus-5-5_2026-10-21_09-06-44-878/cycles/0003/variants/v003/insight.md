# Variant v003 Insight

## Helped

- hit-rate: The only code change is raising `SKETCH_WIDTH_FACTOR` from 4 to 16, which makes each count-min row 4× wider at capacity 2048. Keys now share counters far less often. `estimate_hashed` takes the minimum across rows, so this removes collision overcount from both sides of the strict-`>` contest. Cold newcomers no longer pick up borrowed counts and push out real residents. Cold victims in the 8-node `select_victim` window no longer look warm enough to block deserving newcomers. The gain is clearly larger than the parent's victim-sampling gain. That means some of the "stale-high" estimates the v001 insight blamed on missing aging were actually collision noise, not real past popularity.

## Hurt

## Takeaway

On workload-a, the count-min sketch at 4×capacity width was noisy enough to flip admission decisions, so widening it is a real hit-rate lever and not a no-op. This sketch never ages, so its counters keep growing over the whole trace and each collision adds more error as the run goes on. That makes width matter more here than it would in a sketch that resets. The cost is a fixed 4× larger counter table, which `sketch_heap_bytes()` reports truthfully. This evaluation measured only hit-rate, so the memory cost and any cache-miss latency from the larger table were not scored. Compare future aging/reset hypotheses against this wider-sketch baseline, so their gains are not confused with collision cleanup this variant already captured.
