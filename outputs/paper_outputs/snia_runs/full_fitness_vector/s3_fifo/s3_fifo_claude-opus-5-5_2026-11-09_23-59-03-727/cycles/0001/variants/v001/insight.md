# Variant v001 Insight

## Helped

- hit-rate (hm_0 at both sizes, web_0 at 16384): giving main ~90% of capacity and a matching larger ghost queue kept more reused keys resident, and the 10% small queue still filtered one-hit wonders on these traces.
- per-op-time: get and put both got much faster, but the code change can't explain it. `get` still scans small and main chained together, which is the same total length. `put`'s ghost scan is O(main_size), which got slightly longer. Treat this as drift in the benchmark environment, not something the partition change caused, and re-measure before crediting it.

## Hurt

- hit-rate (web_0 at 131072): with a large cache, web_0's reuse distances seem to need a longer probation window. The smaller small queue sends entries to the ghost queue before their second reference, and this cancels out the gains elsewhere, so the summary is flat.

## Takeaway

On MSR, moving the small-queue share from 25% to 10% shifts hit rate between traces and sizes without improving the summary. hm_0 likes a bigger main queue, while web_0 at large capacity loses from the shorter probation window. That points to a trace-dependent best ratio, so a fixed ratio like the paper's 10% doesn't beat 25% on its own. Nothing else in the code changed (metadata layout and movement rules are the same), so the per-op-time gap here is a measurement artifact and not a property of this variant.
