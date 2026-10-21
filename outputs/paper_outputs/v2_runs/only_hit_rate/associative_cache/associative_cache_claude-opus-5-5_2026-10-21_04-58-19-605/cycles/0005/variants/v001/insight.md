# Variant v001 Insight

## Helped

(No measured dimension improved. `hit-rate` is the only dimension in the fitness vector, and it went down.)

## Hurt

- hit-rate: Growing `GHOST_LEN` from 4 to 16 was the only code change. The admission rule, `CLOCK_MAX`, the LIP hand placement and the linear-scan lookup are all unchanged. So the drop comes from ring capacity alone. With the larger ring, more newcomers match a ghost entry: keys that were evicted 5 to 16 bucket evictions ago now also get counter 1. A newcomer admitted with counter 0 sits under the hand and is evicted at once on the bucket's next miss. A newcomer with counter 1 is only decremented, and the hand moves past it. The next victim is then a resident that has already reached zero. The long-distance returners that the bigger ring now catches mostly do not get hit within that extra pass. Their protection therefore costs evictions of residents that are worth more. The result fell below the parent's figure, which was the threshold H01 set, so the hypothesis is refuted.

## Takeaway

On workload-a, most of the value of ghost-hit admission comes from keys evicted very recently, within about the last 4 evictions in their bucket. Stretching the ring to one entry per slot turns the counter-1 bonus from a selective signal into a diluted one, and the extra protected newcomers push out better residents. The parent's 4-entry ring is at or above the useful size. The next experiment should shrink the ring (for example to 2 or 3) rather than grow it. The miss path also now scans 16 entries instead of 4, which adds unmeasured per-op cost for no gain.
