# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0005's v002 (the age-gated ghost path). v001 left freq-1 ghosts for main victims that had been hit. v002 gave keys that saturated freq in small one protected lap in main. v003 cut the freq-0 recency window to half a turnover. v001 won with an essentially flat summary: small gains at the large size, a slight loss on web_0 at the small size. It is the first change to move web_0 at the large size at all, but the gain is marginal. No variant clearly beat the parent.

## Patterns

- The ghost-age axis is now closed. A 2× window lost last cycle and a ½× window lost this cycle, so one turnover is a local optimum. The returnees in the second half of the window turned out to be real reuse on hm_0, not web_0 pollution.
- Everything that feeds the shared ghost FIFO or its stamp clock interacts with the age gate. v001's main-victim ghosts took ghost space and shrank the effective freq-0 window, and that is where its loss at the small size came from. New ghost producers have to be separated from the parent's clock and budget.
- The parent's quick freq-0 ghost returnees are a valuable population, not a weakly evidenced one. Any change in main that moves other entries ahead of them at the tail (v002) costs hit-rate, most of all on hm_0 at the small size. Main's own second chance already rewards keys that keep getting hit, so a head start at admission adds little.
- web_0 at the large size moved slightly only when the change reached main victims (v001, and v002 barely). That fits the idea that its limit lies in main or first admission. But re-qualifying main victims is too small a population to explain the plateau.
- Per-op-time and metadata-size are still missing, for the seventh cycle running.

## Open questions

- If main-victim ghosts get their own budget, or stop advancing the stamp clock, does v001 keep its large-size gains without the small-size loss?
- What limits web_0 at the large size? First-admission filtering of never-seen keys and the small/main split at that size have not been probed yet.

## Continuation

Treat both cycle 0005's v002 and this cycle's v001 as possible parents. Their policies are nearly tied, and v001's mechanism is only worth keeping if it is decoupled. Try at most one decoupled v001 follow-up: a separate ghost budget, or main→ghost inserts that don't advance stamps. Otherwise move to first-admission handling or the small/main split for web_0 at the large size. Drop all ghost-age window tuning, and drop any main-admission freq boost that puts other entries ahead of quick ghost returnees.
