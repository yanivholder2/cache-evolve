# Variant v002 Insight

## Helped

- hit-rate: Ghost re-admission produced a marginal positive nudge, suggesting stash-bypassed re-insertions do occasionally resolve to hits sooner, but the effect is extremely small relative to the change's complexity.

## Hurt

## Takeaway

Adding a 32-entry FIFO ghost set to bypass stash re-entry for recently evicted keys moved hit-rate by only ~0.0002, effectively noise. The stash eviction boundary is not a dominant loss source in this workload: either the keys that cycle through stash eviction do not re-arrive within the ghost set's retention window, or the Front-insertion position chosen for re-admitted items (requiring them to earn MRU before promoting) neutralises most of the latency savings the bypass was meant to deliver. The ghost set is a correct and safe mechanism, but the workload's reuse pattern does not cycle heavily enough at the stash boundary to reward it.
