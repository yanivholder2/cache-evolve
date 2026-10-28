# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): at the large size, 1024 is still under 1% of capacity per round. The 4x step let the window move a bit further toward a recency-friendly share without making the climber unstable, so this cell improved slightly.

## Hurt

- hit-rate (hm_0 @ 16384): this is the risk H01 named, and it happened. A 1024-entry step is 6.25% of capacity per round. With 2% decay per round over a 50_000-get period, the magnitude would need about 340 rounds to decay toward 1. The trace never runs that long, so the climber keeps taking near-full-size steps. Each noisy hit-rate drop then flips the window by a large amount, and hm_0's frequency-heavy main area loses room it needs. This cell had the largest drop, and it is what pulled the summary down.
- hit-rate (web_0 @ 16384): H01's target cell did not improve; it slipped slightly. A faster climber did not find a better window here, so climber speed was not what held web_0 back at the small size either.
- hit-rate (hm_0 @ 131072): essentially flat. The step is small relative to capacity, so behaviour barely changed.

## Takeaway

Raising the step cap to 1024 disproves the step-speed half of the cycle-0005 explanation for web_0@16384. With faster steps and a 75% ceiling that never binds, the small-size web_0 cell still did not improve. Its gap is probably structural (admission or the duel), not a matter of where the window settles. At 16384, a step that big turns the climber into a noise amplifier on hm_0. The step's effect depends on its size relative to capacity, so any future step change should stay at or below about 1.5% of capacity, or decay much faster than 2% per round. Do not retry larger steps. The `climb` debug log from this run can confirm whether `win_cap` at 16384 oscillated rather than settled.
