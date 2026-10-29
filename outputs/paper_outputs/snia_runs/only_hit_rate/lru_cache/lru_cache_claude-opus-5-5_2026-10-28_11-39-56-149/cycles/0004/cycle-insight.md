# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle 0003's v002 (W-TinyLFU with the tightened sketch), and each changed one lever aimed at the small-size web_0 gap. v001 shortened the sketch reset period. v002 added a hill-climbed window with step and sample period capped at absolute values, and kept the admission duel in force while the window grows. v003 let candidates re-referenced in the window skip the duel. v002 was the only Winner and the first variant in the run to improve every trace/size cell, with by far its largest gain on small-size web_0.

## Patterns

- The cycle 0003 diagnosis held: small-size web_0 is limited by window size. Only enlarging the window recovered it in a large way. Sketch aging (v001) and an admission bypass (v003) each moved it only a little.
- Small-size hm_0 depends on strict one-shot filtering. Both levers that loosened admission gave back much of v002's earlier hm_0 gain at tight capacity. v001 did it through faster count decay and doorkeeper clears. v003 did it by letting burst-then-dead blocks through on a single in-window hit. The long reset period and the doorkeeper should be treated as load-bearing.
- Fixing one lever per variant worked. Each result maps to a clear cause, unlike in earlier cycles. The cause of cycle 0003's small-size web_0 drop is now narrowed. The reset period explains little of it, so the doorkeeper's colder first-touch estimates, or simply the window being too small, remain the explanation.
- The lesson about scale-dependent parameters was confirmed from the other direction. Capping the climber's knobs instead of scaling them with capacity removed the large-size loss of cycle 0003's climber. The floor at the old window size protected hm_0.
- The cohort repeated an earlier lesson: hard overrides lose to soft, relative signals. The in-window reuse signal carries real information on web_0, but as an unconditional bypass it costs more than it gains.
- For the fourth cycle, only hit-rate was measured.

## Open questions

- Which cap does the work in v002: the sample-period cap or the step cap? Both test sizes hit the same caps, so their effects can't be separated. How far does the window actually move in each cell?
- With a larger adaptive window in place, does the doorkeeper still cost anything on web_0, or has the climber absorbed that loss?
- Could in-window reuse help as a bounded bonus or tie-breaker inside the duel rather than as a bypass?
- Does the climber's per-access sampling add measurable per-op cost?

## Continuation

Build on cycle 0004's v002 and keep its sketch (long reset, doorkeeper, dedup, wide width), its capped climber, and its window floor. Next, vary the period cap and the step cap one at a time, and log the final window size in each cell. Separately, test in-window reuse as a small duel bonus only. Drop shorter reset periods and any unconditional admission bypass. Getting `metadata-size` and `per-op-time` reported is now four cycles overdue and should come before more complexity is added.
