# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Each variant tested one structural lever named in cycle 0007's continuation. v001, from 0007's v001, decoupled the main-victim ghost and capped it at one small queue's worth of keys. v002, from 0007's v001, shrank small only at the large size and pinned the age window. v003, from 0005's v002, doubled the ghost budget. v003 won, but only narrowly: it gained on hm_0 at the small size, the losses elsewhere nearly cancelled that gain, and it was roughly neutral overall. No lever gained on both traces. Each one traded hm_0 against web_0.

## Patterns

- The two traces disagree on every lever. hm_0 gains from more probation, longer memory or a bigger ghost horizon at a tight capacity. web_0 is hurt by any change that gives privileges to keys returning after a long gap. The exception is main victims that were hit, which return over a horizon closer to `main_size` than to `small_size`.
- The useful information in a ghost record is how long ago the key was seen and where it was evicted from, not just that it was seen. v001 showed that main victims and small victims need ghost horizons of different sizes. v003 showed that records in the extended zone should not get the same privileges as recent ones.
- Cycle 0007 guessed that the shared stamp clock caused 0007-v001's web_0 loss at the small size. Restoring the clock did not recover that loss, so the guess was mostly wrong. Decoupling is still worth keeping, because it protects the freq-0 window.
- The small/main split is a weak lever for web_0 at the large size, and it costs hm_0 more than web_0 gains. So the split does not explain that cell's plateau either. web_0 at the large size has moved only when main victims that were hit were remembered over a horizon of about `main_size`.
- Per-op-time and metadata-size are still not measured, for the eighth cycle running. This matters more now, because v003 doubles ghost metadata.

## Open questions

- Does a decoupled main-victim ghost with a budget near `main_size` keep web_0's large-size gain and v001's protected freq-0 window?
- Does a doubled ghost with tiered privileges help hm_0 at the small size without the web_0 losses? Under tiering, records in the extended zone would give freq-0 returnees the freq-1 small path but would not send freq ≥ 1 returnees straight to main.
- Does a midpoint split, or a window rescaled to the new small, recover hm_0 at the large size? This is a low priority.

## Continuation

Two targeted follow-ups are worth one hypothesis each. First, scale v001's decoupled `main_ghost` budget toward `main_size`. Second, tier v003's extended ghost zone so that old records do not grant direct entry to main. Combining the two, once each works, is the natural next step. Drop small/main split tuning and uniform ghost-budget scaling. Do not retry stamp-clock fixes for web_0's small-size loss. Note: v003 is the nominal winner but is roughly neutral overall, so 0005's v002 is still an equally valid base.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Two variants ran cycle 0008's suggested follow-ups. v001 tiered 0008-v003's doubled ghost. v002 scaled 0008-v001's `main_ghost` budget up to `main_size`. v003 took the first step off the ghost path: it applied a correlated-reference cap on small-queue hits that arrive soon after insertion, built on 0005-v002. v003 won on hm_0, gaining at both sizes. It lost at the small size on web_0 and was flat at the large size there.

## Patterns

- The hm_0 / web_0 trade-off held again, now for a fourth cycle in a row. Every lever helped one trace at the other's expense, and each variant's gain showed up only at the small size, or mostly there.
- Both of 0008's follow-ups failed. Tiering removed direct-to-main entry for old freq ≥ 1 ghosts, and it turned out that privilege was part of hm_0's small-size gain. Ghost age alone does not separate useful late returnees from noisy ones. A three-times-larger `main_ghost` left both large-size cells unchanged. That refutes 0008's claim that a horizon of about `main_size` for hit main victims explains web_0's large-size plateau.
- v003's gain comes from deferring main admission, not rejecting it. A key whose small-queue hits all fall in its insertion burst keeps freq 1 into the ghost. The freq ≥ 1 ghost→main path then admits it on a later, spaced return. hm_0 has burst-only keys that pollute a full main. On web_0, a quick reuse after insertion is real evidence.
- web_0 at the large size has now resisted ghost routing, ghost age, ghost budgets, the split, main-victim memory and burst filtering. Cycle 0001's web_0 pollution does not come from burst-only keys.
- Per-op-time and metadata-size are still not measured, for the ninth cycle running.

## Open questions

- Would a narrower correlated-reference window (a larger `CRP_DIVISOR`) keep hm_0's gain and spare web_0's quick reuses?
- For web_0 at the large size, two candidates are left. Which main victims get ghosted at all (the `main_hit` gate)? How do main-ghost returnees get admitted (currently freq 0, straight into main)? Or is that cell simply at its ceiling for this policy family?

## Continuation

Use 0009-v003 (the correlated-reference cap on 0005-v002) as the policy base. Spend one hypothesis on narrowing its window. Drop ghost tiering by age, `main_ghost` budget scaling and uniform ghost-budget scaling, because those axes are exhausted. Spend at most one probe on web_0 at the large size, through the `main_hit` gate or the freq given to main-ghost returnees. If that probe also fails, treat the cell as a plateau and stop targeting it.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
