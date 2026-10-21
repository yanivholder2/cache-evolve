# Variant v003 Insight

> Note: I could not open the overlay source or manifest in this session. Only Read was available, so I could not list directories, and the file paths I guessed did not exist. The mechanism below comes from H03's stated change: ghost hits with age ≤ 1 epoch go straight to the normal-tier target bucket, using the promotion placement path. I did not check the diff itself. That matters here, because an unchanged result is also what a change that never fires, or always falls back, would produce.

## Helped

## Hurt

- hit-rate: Essentially unchanged; the tiny drop is within noise. Sending very-recent ghost hits straight to target neither freed useful stash capacity nor wasted target slots. Most likely those keys were going to earn their stash→target promotion anyway. Skipping probation only changes when they arrive, not whether they survive. Each direct admission also demotes the displaced target resident to stash MRU, so the stash slot it saved is filled again at once. Stash occupancy, and so "who reaches the stash", barely changes.

## Takeaway

On this LIRS trace at size 2048, the stash probation round trip costs almost nothing for ghost hits with age ≤ 1 epoch. Under the existing threshold they promote reliably, and the demotion that comes with any target insert cancels the stash slot that skipping probation was supposed to free. So "straight to target" is roughly neutral, not a lever. The stash-mediated path holds up as no worse and is simpler. The 0008/0009 goal of changing who reaches the stash will need a change that does not push a demoted resident back into the stash. Before closing this out, check two things in the eval/instrumentation: how often the age ≤ 1 branch actually fired, and how often it fell back to stash MRU. A near-zero count would mean the hypothesis was not really tested.
