# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005's v002 (the age-gated ghost path), and all three changed its ghost or small routing. v001 restacked the one-time relap, this time ghosting failed relaps at freq 1. v002 widened the freq-0 recency window to two small-queue turnovers. v003 added an age ceiling on freq ≥ 1 returnees. v003 won, but only because it moved the least: it rose slightly on hm_0 at the large size and dipped slightly in every other cell. None of the three cleanly beat the parent, so this cycle mapped where the policy's boundaries are rather than finding a new gain.

## Patterns

- The parent's ghost gate is close to a local optimum on all three axes probed. Stacking a second reward for one reuse (v001), loosening the recency floor (v002) and tightening admission for freq-1 returnees (v003) each gave back hit-rate on web_0, which is the trace the parent's gate was tuned against.
- The two signals do different jobs. Ghost age is useful as a floor on freq-0 returnees, and about one small-queue turnover is the right boundary: going to two turnovers lets in web_0's late one-off rereferences. Age adds little as a ceiling on freq-1 returnees, because a reuse in small is still good evidence late in the ghost window.
- Once failed relaps keep their freq-1 ghost signal, the relap duplicates what the ghost path already does, and it costs small-queue capacity. The cycle 0005 relap gain on web_0 at the large size came from the old freq-0 base and did not carry over.
- web_0 at the large size has now resisted every ghost and small routing change, whether it loosened or tightened admission. Whatever limits that cell, it isn't how ghost returnees are routed.
- Every small-queue re-admission (relap, re-routed returnee) pushes another resident out of small. At the small cache size this displacement keeps costing more than whatever main pollution it avoids.
- Per-op-time and metadata-size are still absent from the evaluation, for the sixth cycle running.

## Open questions

- What actually limits web_0 at the large size: main's eviction order, the small/main split at that size, or first-admission filtering of keys never seen before?
- Would a window narrower than one turnover sharpen the web_0 / hm_0 split, or is one turnover already at the boundary?
- Is hm_0's modest large-size gain under v003 real, or is it noise?

## Continuation

Keep cycle 0005's v002 policy, with the age window at 1×, as the policy base. Drop relap stacking, windows wider than one turnover, and age ceilings on freq ≥ 1 returnees; the ghost-path routing axis looks exhausted. If policy work continues, move off the ghost path and target main's eviction or first-admission handling for web_0 at the large size, and try at most one narrower-window probe. Otherwise, port the pending cost-only changes and push for per-op-time and metadata-size to be measured.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
