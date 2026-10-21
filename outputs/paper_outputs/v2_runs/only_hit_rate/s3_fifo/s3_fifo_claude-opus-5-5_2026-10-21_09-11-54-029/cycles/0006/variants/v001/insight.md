# Variant v001 Insight

## Helped

(No measured dimension improved. `hit-rate` is the only dimension in the fitness vector, and it moved the wrong way.)

## Hurt

- hit-rate: v003 had one shared ghost of `main_size` live entries. This variant splits it into `ghost_small` (cap `main_size`) and `ghost_main` (cap `main_size / 2`), so the total ghost window grows to about 1.5× `main_size`. Every extra ghost hit in `put` skips the small-queue filter (`freq > 1` to graduate) and lands at the front of main. Each of those admissions forces an `evict_main` when main is full. More keys of uncertain value enter main, and resident main entries get pushed out sooner. On workload-a that churn cost more than the restored small-reject evidence recovered.

## Takeaway

H01 assumed that the shared-budget "dilution" in v003 was hiding part of the main-victim gain. This variant shows the opposite on workload-a. When main victims and small rejects competed for one `main_size` window, the total number of ghost-driven bypasses into main stayed capped, and that cap was protective. Removing it by giving each source its own budget made admission into main more permissive. Re-admission still uses the same front-of-main, freq-0 path, so the extra ghost hits mostly displaced residents.

There is a second possible cause. If main victims took more than half of v003's shared window, the fixed `main_size / 2` cap actually shrank their evidence. One trace at one size can't separate this from the cause above.

Ghost capacity is not free evidence in this design. Every ghost hit costs a main eviction. The next steps are either a smaller combined budget, or a way to make returning ghost keys earn their place (for example, re-entering through small or at the tail of main), rather than adding more ghost entries.
