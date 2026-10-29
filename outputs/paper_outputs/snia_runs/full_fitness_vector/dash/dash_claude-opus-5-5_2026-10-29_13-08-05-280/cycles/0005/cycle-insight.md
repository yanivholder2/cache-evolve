# Cycle 0005 Insight

## Cohort outcome

All three variants branched separately from cycle 0004 v003, and all three are Winners. v001 merged in the 24-byte Item with a per-segment tick and added the small-cache size gate. v002 replaced the full-probing cascade with a direct two-bucket swap. v003 made the probing guard strict, so any non-hottest hit resets the count. Two variants gave clean gains at no hit-rate cost: v001 improved metadata-size and latency, and v002 cut the get tail. v003 bought a small hit-rate gain with clear get-latency costs.

## Patterns

- Removing work beats deferring it, and this held again. v002's swap removed the cascade outright and gave back v003's get-tail cost. Hit-rate was identical to the last digit in every cell. *Where* the cascade sent displaced entries never mattered. Only *when* promotion fires does.
- Contention, not layout, caused cycle 0004's 24-byte regression. With a per-segment tick, the smaller Item made puts and the get tail faster, probably because the slow paths touch fewer cache lines.
- Any check on the common hit path shows up in median get. This is the third cycle with this lesson. v003's reset sits on the most common hit path, so median get rose even though the reset almost never fires.
- The stash-side size gate is now mostly redundant. Because the target-side probing guard already fixed the small-cache slip, the gate adds only a negligible small-cache gain.
- The guarded full-probing arm seems to fire rarely on these traces. v002's identical hit-rate suggests this, so the remaining tails likely come from other paths: stash→target moves or put eviction. This sits uneasily with v003's hit-rate gain on that same arm, and the two results need reconciling.
- Noise is still unmeasured. v002's and v003's put tails moved with `put` untouched. None of the three variant insights could inspect the overlay source, so their mechanisms are inferred from the hypotheses.

## Open questions

- Do v001 and v002 combine cleanly? They touch separate paths, so the gains should add.
- Can v003's strict reset be moved off the common hit path while keeping its web_0-large gain? One option is to check staleness lazily when the hottest hit happens, for example by comparing ticks.
- How often does the full-probing arm actually fire? Which paths now dominate get and put p99?
- How large is the per-op-time noise? A re-run of an identical baseline is still missing after four cycles of asking.

## Continuation

Merge v001 (24-byte Item, per-segment tick) with v002 (direct swap) as the new baseline. Consider dropping the size gate as redundant complexity. Keep v003's strict-guard idea only if the reset can leave the common hit path, for example as a lazy tick-based staleness check on the rare hottest arm. Otherwise drop it. Next, target the tail paths that remain, stash→target and put eviction. Re-run the baseline before acting on any latency-only change.
