# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Halving every counter after each `capacity × 256` increments (with `additions` halved too, Caffeine-style) left hit-rate essentially unchanged, with a slip small enough to be noise. Each reset cuts the estimates of real long-term-hot residents just as much as stale ones. So in the strict-`>` contest against the minimum-frequency key from the 8-entry `select_victim` window, newcomers win no more often than before, and some lose a residency they would otherwise have kept until counts rebuild.

## Takeaway

This is the clean re-test the parent's insight asked for: aging on the 16×-width sketch with no collision noise. At a very long period, aging gives no hit-rate on workload-a. That supports the parent's reading that the "stale-high" blocking earlier aging tries were aimed at was collision overcount, not real ancient popularity. Treat aging as settled-negative for workload-a on the wide sketch. Spend future hypotheses elsewhere, such as the admission window and contest, or victim selection, rather than shorter or longer aging periods.
