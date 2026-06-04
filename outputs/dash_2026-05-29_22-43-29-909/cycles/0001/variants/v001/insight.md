# Variant v001 Insight

## Helped

- per_op_time: Promotions now land via an O(1) `push` to the Vec's Back instead of the baseline's `Front` insert (an index-0 shift of the whole bucket), so the costly path off the hot promotion route disappears and put tail latency drops.

## Hurt

- hit_rate: Biasing every promotion to the protected MRU end keeps recently-touched-but-soon-cold keys resident, and on this loop/scan-heavy LIRS trace pure-recency retention is exactly what gets punished — so the recency bet H01 made does not convert into hits.

## Takeaway

Forcing promoted items to the MRU end realizes a recency-favoring policy, but on a LIRS scan/loop workload recency is the wrong signal — reuse-distance or frequency awareness is what wins, so the hot-promotion bet underdelivers. Note the change landed as a blanket "ignore `InsertPosition`, always Back" in `put_at` rather than a targeted promotion-path flip; equivalent under ClassicLRU only because new-key `put` already inserted at Back, but it couples bucket movement to that one policy's geometry.
