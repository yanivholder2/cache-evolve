# Cycle 0003 Insight

## Cohort outcome

Three single-axis changes were tried: u32 list indices in place of `Option<usize>` links (v001), a two-candidate victim sample (v002), and a tie-admitting `>=` comparison (v003). v001 and v002 were kept as Winners. v003 was not. All three cost per-op time, and none gave a real hit-rate gain.

## Patterns

- The only change that moved a headline dimension was v001's layout shrink. It cut Node size with hit-rate unchanged, but it gave back per-op latency. The working set is small, so the locality benefit did not appear. Extra index conversion and sentinel checks seem to cost more than the saved cache lines.
- Both admission-quality changes (wider victim sample, looser tie rule) failed to lift hit-rate. The wider sample was marginal and probably noise. Admitting ties lowered hit-rate. Strict greater-than admission with the LRU head as victim again looks well suited to workload-a.
- Frequencies on this trace are low and often tied. Ties are common, so the tie rule matters, and loosening it removes protection for resident entries.
- Latency rose even where the hot path was untouched (v002's get, v003's one-operator edit). Some of the per-op regressions are probably measurement noise or code-layout effects, so small time deltas are unreliable evidence.
- Two of the insights were written without a source diff, so some of their attributions are inferred.
- A cheap victim-selection or comparison tweak is now ruled out as a hit-rate lever, adding to the earlier failures of aging and a window.

## Open questions

- Is v001's size gain worth its latency cost? Does the cost come from NIL checks and widening, or is it noise?
- Can the per-op regression be recovered, for example with unchecked indexing or a layout that avoids repeated conversion?
- Is there any signal that separates tied frequencies without adding hot-path work?
- How much of the observed latency variance is noise, and does it need repeat runs?

## Continuation

Keep v001 as the compact-layout parent, but aim next at winning back its per-op time (fewer conversions, tighter hot-path code) and combine it with cycle 0002's v001 hashing speedup and the compact sketch. Drop victim sampling and tie-admitting comparisons. Hit-rate gains need a genuinely new signal (a doorkeeper or sketch accuracy), not tweaks to the admission comparison or victim choice. Use a larger or repeated measurement before trusting small latency deltas.
